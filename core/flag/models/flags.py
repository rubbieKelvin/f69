import uuid
from django.db import models

# TODO: I stopped here. i need to find a way to make this rule based.
# If i cant do that, i need to just do segment based serving

class Flag(models.Model):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    feature = models.ForeignKey(
        "flag.Feature", on_delete=models.CASCADE, related_name="values"
    )
    # TODO: remove null = true after we clean up the db
    environment = models.ForeignKey("flag.Environment", on_delete=models.CASCADE, null=True)
    enabled = models.BooleanField(default=False)
    # segments = models.ManyToManyField("flag.Segment", blank=True, help_text="These are the segments ")
