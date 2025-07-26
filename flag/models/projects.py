import uuid
import typing as t
from django.db import models
from django.conf import settings
from django.contrib.auth.models import AbstractBaseUser


class Project(models.Model):
    """A user can create a project where flags needs to be managed"""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    name = models.CharField(max_length=225)
    description = models.CharField(max_length=1000, blank=True, default="")
    author = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)
    is_deleted = models.BooleanField(default=False)

    def __repr__(self):
        name = self.name
        user = t.cast(AbstractBaseUser, self.author)
        email = user.get_username()

        return f"Project(name={name}, author={email})"

    def __str__(self):
        return self.name
