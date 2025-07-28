from django import forms
from flag.models import ProjectClientSecret


class SecretCreateForm(forms.ModelForm):
    """Form for creating a new API secret"""
    
    class Meta:
        model = ProjectClientSecret
        fields = ['name']
        
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        
        # Add CSS classes and attributes to form fields
        self.fields['name'].widget.attrs.update({
            'class': 'form-input',
            'placeholder': 'Enter a descriptive name for this secret (e.g., "Production API", "Mobile App")',
            'required': True,
        })
        
        # Add help text
        self.fields['name'].help_text = "A descriptive name to help you identify this secret later"
    
    def clean_name(self):
        """Validate that name is not empty after stripping"""
        name = self.cleaned_data.get('name', '').strip()
        if not name:
            raise forms.ValidationError("Secret name cannot be empty.")
        return name 