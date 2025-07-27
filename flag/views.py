from ninja import NinjaAPI
from django.contrib.auth.hashers import check_password
from django.http import HttpRequest
from ninja.throttling import AuthRateThrottle
from ninja.security import APIKeyHeader
from .models.projects import Project, ProjectClientSecret


def authenticator(request: HttpRequest):
    # Note: the client id is the project author's id
    client_id = request.headers.get("f69_client_id")
    project_id = request.headers.get("f69_project_id")
    project_secret = request.headers.get("f69_project_secret")

    try:
        project = Project.objects.filter(author__id=client_id).get(id=project_id)
    except Project.DoesNotExist:
        return None

    # CHeck for project secrets
    secrets = ProjectClientSecret.objects.filter(project=project)
    secret_hash = [secret.secret_hash for secret in secrets]

    for secret in secret_hash:
        if check_password(project_secret, secret):
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


@api.post("/flags")
def cook_feature_flags(
    request: HttpRequest
):
    """
    Cook feature flags for the given request. we'll need the
    client_id (in header 'X-f69-client-id'),
    project_id (in path params),
    and project_secret (in header 'X-f69-client-secret').
    """
    print(request.user)
    return []
