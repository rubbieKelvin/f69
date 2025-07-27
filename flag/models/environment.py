import uuid
from django.db import models


class Environment(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    name = models.CharField(max_length=225)
    description = models.CharField(max_length=1000, blank=True, default="")
    project = models.ForeignKey(
        "flag.Project", on_delete=models.CASCADE, related_name="environments"
    )
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)
