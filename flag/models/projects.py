import secrets
import uuid
import typing as t
from django.db import models
from django.conf import settings
from django.contrib.auth.models import AbstractBaseUser
from django.contrib.auth.hashers import make_password
from django.db.models.manager import Manager

if t.TYPE_CHECKING:
    from .access import ProjectAccess
    from .features import Feature
    from .entities import Entity
    from .segments import Segment
    from .environment import Environment

class Project(models.Model):
    """A user can create a project where flags needs to be managed"""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    name = models.CharField(max_length=225)
    description = models.CharField(max_length=1000, blank=True, default="")
    author = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)
    is_deleted = models.BooleanField(default=False)

    # related fields
    access: "Manager[ProjectAccess]"
    features: "Manager[Feature]"
    entities: "Manager[Entity]"
    segments: "Manager[Segment]"
    environments: "Manager[Environment]"

    def __repr__(self):
        name = self.name
        user = t.cast(AbstractBaseUser, self.author)
        email = user.get_username()

        return f"Project(name={name}, author={email})"

    def __str__(self):
        return self.name


class ProjectClientSecret(models.Model):
    id = models.UUIDField(default=uuid.uuid4, primary_key=True)
    name = models.CharField(max_length=225)
    secret_hash = models.CharField(max_length=200)
    project = models.ForeignKey("flag.Project", on_delete=models.CASCADE)
    author = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    created_at = models.DateTimeField(auto_now_add=True)

    @staticmethod
    def new(
        *, name: str, author: AbstractBaseUser, project: "Project"
    ) -> tuple[str, "ProjectClientSecret"]:
        secret = secrets.token_hex(32)

        return secret, ProjectClientSecret.objects.create(
            secret_hash=make_password(secret),
            project=project,
            author=author,
            name=name,
        )
