from django.urls import path
from .views import cook_feature_flags

urlpatterns = [
    path("/<id>", cook_feature_flags, name="generate-feature-flags"),
]
