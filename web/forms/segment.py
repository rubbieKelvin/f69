from django import forms
from flag.models import Segment, Entity


class SegmentCreateForm(forms.ModelForm):
    """Form for creating new segments"""

    class Meta:
        model = Segment
        fields = ["name", "description"]
        widgets = {
            "name": forms.TextInput(
                attrs={"class": "form-input", "placeholder": "Enter segment name"}
            ),
            "description": forms.Textarea(
                attrs={
                    "class": "form-textarea",
                    "placeholder": "Describe this segment (optional)",
                    "rows": 4,
                }
            ),
        }
        help_text = {
            "name": "A unique name for this segment",
            "description": "Optional description of what this segment represents",
        }

    def clean_name(self):
        name = self.cleaned_data["name"]
        # You could add custom validation here if needed
        return name


class SegmentEditForm(forms.ModelForm):
    """Form for editing segments with entity management"""

    entities = forms.ModelMultipleChoiceField(
        queryset=Entity.objects.none(), # initialy set this to none. we'd update this with the project's entities later
        widget=forms.CheckboxSelectMultiple(attrs={"class": "entity-checkbox-list"}),
        required=False,
        help_text="Select entities to include in this segment",
    )

    class Meta:
        model = Segment
        fields = ["name", "description", "entities"]
        widgets = {
            "name": forms.TextInput(
                attrs={"class": "form-input", "placeholder": "Enter segment name"}
            ),
            "description": forms.Textarea(
                attrs={
                    "class": "form-textarea",
                    "placeholder": "Describe this segment (optional)",
                    "rows": 4,
                }
            ),
        }
        help_text = {
            "name": "A unique name for this segment",
            "description": "Optional description of what this segment represents",
        }

    def __init__(self, *args, project=None, **kwargs):
        super().__init__(*args, **kwargs)
        if project:
            # Only show entities from the current project
            entities_field = self.fields["entities"]
            setattr(entities_field, "queryset", project.entities.all().order_by("name"))

    def clean_name(self):
        name = self.cleaned_data["name"]
        # You could add custom validation here if needed
        return name
