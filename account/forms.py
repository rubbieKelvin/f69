from .models import User
from django.contrib.auth.forms import UserCreationForm as UCrF, UserChangeForm as UChF


class UserCreationForm(UCrF):

    class Meta:
        model = User
        fields = ("email",)


class UserChangeForm(UChF):

    class Meta:
        model = User
        fields = ("email",)
