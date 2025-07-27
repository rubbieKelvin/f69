from django import forms
from flag.models import Feature


class FeatureCreateForm(forms.ModelForm):
    """Form for creating a new feature flag"""
    
    class Meta:
        model = Feature
        fields = ['name', 'slug', 'description']
        
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        
        # Add CSS classes and attributes to form fields
        self.fields['name'].widget.attrs.update({
            'class': 'form-input',
            'placeholder': 'Enter feature name (e.g., "New Dashboard")',
            'required': True,
        })
        
        self.fields['slug'].widget.attrs.update({
            'class': 'form-input',
            'placeholder': 'Enter feature slug (e.g., "new-dashboard")',
            'required': True,
        })
        
        self.fields['description'].widget.attrs.update({
            'class': 'form-textarea',
            'placeholder': 'Describe what this feature does and when it should be used...',
            'rows': 4,
        })
        
        # Add help text
        self.fields['name'].help_text = "A human-readable name for your feature flag"
        self.fields['slug'].help_text = "A unique identifier used in your code (lowercase, hyphens only)"
        self.fields['description'].help_text = "Optional description to help your team understand this feature"
    
    def clean_slug(self):
        """Validate that slug contains only lowercase letters, numbers, and hyphens"""
        slug = self.cleaned_data.get('slug')
        if slug and not slug.replace('-', '').replace('_', '').isalnum():
            raise forms.ValidationError(
                "Slug can only contain lowercase letters, numbers, hyphens, and underscores."
            )
        return slug.lower() if slug else slug 