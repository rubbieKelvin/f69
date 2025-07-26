import uuid
from django.db import models
from django.conf import settings


class Role(models.TextChoices):
    ADMIN = "ADM", "Admin"
    DEVELOPER = "DEV", "Developer"


class ProjectAccess(models.Model):
    id = models.UUIDField(editable=False, primary_key=True, default=uuid.uuid4)
    user = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    role = models.CharField(max_length=3, choices=Role, default=Role.DEVELOPER)
    project = models.ForeignKey(
        "flag.Project",
        on_delete=models.CASCADE,
        related_name="access",
    )
    created_at = models.DateTimeField(auto_now_add=True)
