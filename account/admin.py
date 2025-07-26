from django.contrib import admin
from .models import NewUser
from django.contrib.auth.admin import UserAdmin
# Register your models here.

from .forms import NewUserCreationForm, NewUserChangeForm



class NewUserAdmin(UserAdmin):
    add_form = NewUserCreationForm
    form = NewUserChangeForm
    model = NewUser
    list_display = ("id","email","first_name","last_name", "is_staff", "is_active",)
    list_filter = ("email","first_name","last_name", "is_staff", "is_active",)
    fieldsets = (
        (None, {"fields": ("email","first_name","last_name" "password")}),
        ("Permissions", {"fields": ("is_staff", "is_active", "groups", "user_permissions")}),
    )
    add_fieldsets = (
        (None, {
            "classes": ("wide",),
            "fields": (
                "email","first_name","last_name","password1", "password2", "is_staff",
                "is_active", "groups", "user_permissions"
            )}
        ),
    )
    search_fields = ("email","first_name","last_name")
    ordering = ("-date_joined",)


admin.site.register(NewUser, NewUserAdmin)