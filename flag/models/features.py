import uuid
from django.db import models
from django.conf import settings


class Feature(models.Model):
    """This is the feature to be flagged. it belongs directly to a project"""

    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    name = models.CharField(max_length=225)
    slug = models.SlugField(max_length=300)
    description = models.CharField(max_length=1000, blank=True, default="")
    project = models.ForeignKey("flag.Project", on_delete=models.CASCADE)
    author = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        unique_together = ("project", "slug")