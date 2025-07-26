from django import forms
from flag.models import Project

class ProjectCreateForm(forms.ModelForm):
    """Form for creating a new project"""
    
    class Meta:
        model = Project
        fields = ['name', 'description']
        widgets = {
            'name': forms.TextInput(attrs={
                'placeholder': 'Enter project name',
                'class': 'form-input'
            }),
            'description': forms.Textarea(attrs={
                'placeholder': 'Enter project description (optional)',
                'rows': 4,
                'class': 'form-textarea'
            }),
        }
        
    def clean_name(self):
        name = self.cleaned_data.get('name')
        if name:
            name = name.strip()
            if len(name) < 3:
                raise forms.ValidationError("Project name must be at least 3 characters long.")
        return name
