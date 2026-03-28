import uuid
from django.db import models


class Segment(models.Model):
    """A segment is just a group of entities.
    they belong to a project and they are used to group entities together to apply to a flag value
    """

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    name = models.CharField(max_length=225)
    description = models.CharField(max_length=1000, blank=True, default="")
    project = models.ForeignKey(
        "flag.Project", on_delete=models.CASCADE, related_name="segments"
    )
    entities = models.ManyToManyField("flag.Entity", blank=True)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)
