from django.shortcuts import render, redirect
from django.http import HttpRequest, HttpResponse
from django.contrib.auth.mixins import LoginRequiredMixin
from django.contrib import messages
from django.core.exceptions import ValidationError
import typing as t
from account.models import User
from flag.models import Project, Feature


def feature_detail_view(request: HttpRequest, project_id: str, feature_id: str) -> HttpResponse:
    """View to display individual feature flag details"""
    user = t.cast(User, request.user)

    # Get the project and verify access
    try:
        project = Project.objects.filter(access__user__id=user.id).get(
            id=project_id
        )
    except Project.DoesNotExist:
        messages.error(request, "Project not found or access denied")
        return redirect("home")
    except ValidationError:
        messages.error(request, "Invalid project ID")
        return redirect("home")

    # Get the feature and verify it belongs to this project
    try:
        feature = project.features.get(id=feature_id)
    except Feature.DoesNotExist:
        messages.error(request, "Feature not found")
        return redirect("project-features", id=project_id)
    except ValidationError:
        messages.error(request, "Invalid feature ID")
        return redirect("project-features", id=project_id)

    context = {
        "project": project,
        "feature": feature,
        "current_tab": "features",
    }

    return render(request, "project/feature-detail.html", context) 