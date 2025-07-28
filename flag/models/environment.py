import uuid
import typing as t
from django.db import models

if t.TYPE_CHECKING:
    from .projects import Project

class Environment(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    name = models.CharField(max_length=225)
    slug = models.SlugField(max_length=255)
    description = models.CharField(max_length=1000, blank=True, default="")
    project: 'models.ForeignKey[Project]' = models.ForeignKey(
        "flag.Project", on_delete=models.CASCADE, related_name="environments"
    )
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    def __str__(self):
        return f'{self.project.name}/{self.name}'
    
    class Meta:
        unique_together = ('slug', 'project')