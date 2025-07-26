import uuid
from django.db import models


class Flag(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    feature = models.ForeignKey("flag.Feature", on_delete=models.CASCADE)
    enabled = models.BooleanField(default=False)
