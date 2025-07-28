import uuid
from django.db import models
from django.conf import settings


class Entity(models.Model):
    """
    This would be served to us by the client. We need a way to keep track of users/entities in a project.
    Entities might be an organisation, a user, or a resource. but typically a user.
    """

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    name = models.CharField(max_length=225)
    tag = models.SlugField(
        max_length=50,
        help_text="This is the entity type, could be anything described by the client; eg: user, organisation, workspace",
    )
    external_id = models.UUIDField(help_text="This is the entity id from the client")
    project = models.ForeignKey(
        "flag.Project", on_delete=models.CASCADE, related_name="entities"
    )
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)
    vars = models.JSONField(null=True, blank=True)

    class Meta:
        verbose_name_plural = "entities"
        unique_together = [['external_id', 'project']]
