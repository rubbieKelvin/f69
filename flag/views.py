from ninja import NinjaAPI
from ninja.errors import HttpError
from ninja.throttling import AuthRateThrottle

from django.http import HttpRequest
from django.db.transaction import atomic
from django.core.exceptions import ValidationError
from django.contrib.auth.hashers import check_password

import typing as t
from pydantic import BaseModel

from .models.projects import Project, ProjectClientSecret
from .models import Entity, Environment


def authenticator(request: HttpRequest):
    # Note: the client id is the project author's id
    client_id = request.headers.get("X-F69-Client-Id")
    project_id = request.headers.get("X-F69-Project-Id")
    project_secret = request.headers.get("X-F69-Project-Secret")

    try:
        project = Project.objects.filter(author__id=client_id).get(id=project_id)
    except Project.DoesNotExist:
        return None

    # CHeck for project secrets
    secrets = ProjectClientSecret.objects.filter(project=project)
    secret_hash = [secret.secret_hash for secret in secrets]

    for secret in secret_hash:
        if check_password(project_secret, secret):
            setattr(request, "project", project)
            return project

    return None


api = NinjaAPI(
    title="f69 Flag API",
    version="0.1.0",
    description="API for managing feature flags",
    csrf=False,
    throttle=[AuthRateThrottle("1000/s")],
    auth=authenticator,
)


class EntityData(BaseModel):
    ref_id: str
    name: str
    tag: str
    variables: dict[str, t.Any] | None


class FlagsBody(BaseModel):
    environment: str
    entities: list[EntityData]


class ErrorResponse(BaseModel):
    message: str


@api.post("/flags", response={404: t.Any, 200: list[str], 401: t.Any})
@atomic
def cook_feature_flags(request: HttpRequest, body: FlagsBody):
    """
    Cook feature flags for the given request. we'll need the
    client_id (in header 'X-f69-client-id'),
    project_id (in path params),
    and project_secret (in header 'X-f69-client-secret')
    """
    project: Project | None = getattr(request, "project", None)

    if project is None:
        # return 404, ErrorResponse(message="Project not foiund")
        raise HttpError(404, "Project not found")

    environment: Environment = get_environment(project, body.environment)

    try:
        entities: list[Entity] = manage_entities(project, body.entities)
    except ValidationError:
        raise HttpError(401, "Invalid entity data")

    return []


def manage_entities(project: Project, elist: list[EntityData]) -> list[Entity]:
    result: list[Entity] = []

    for e in elist:
        try:
            entity = Entity.objects.get(project=project, external_id=e.ref_id)
            updated = (
                entity.name != e.name
                or entity.tag != e.tag
                or entity.vars != e.variables
            )

            if entity.name != e.name:
                entity.name = e.name

            if entity.tag != e.tag:
                entity.tag = e.tag

            if entity.vars != e.variables:
                entity.vars = t.cast(t.Any, e.variables)

            if updated:
                entity.save()

            result.append(entity)
        except Entity.DoesNotExist:
            entity = Entity.objects.create(
                name=e.name,
                tag=e.tag,
                external_id=e.ref_id,
                project=project,
                vars=e.variables,
            )

            result.append(entity)

    return result


def get_environment(project: Project, env: str) -> Environment:
    try:
        return Environment.objects.get(project=project, slug=env)
    except Environment.DoesNotExist:
        raise HttpError(401, "Environment does not exist")
