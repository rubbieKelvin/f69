import uuid
from django.db import models
from django.conf.global_settings import AUTH_USER_MODEL


class Project(models.Model):
    """A user can create a project where flags needs to be managed"""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    name = models.CharField(max_length=225)
    description = models.CharField(max_length=1000, blank=True, default="")
    author = models.ForeignKey(AUTH_USER_MODEL, on_delete=models.CASCADE)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)
