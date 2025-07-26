from django.views import View
from django.http import HttpRequest, HttpResponse
from django.shortcuts import render, redirect
from django.contrib.auth.mixins import LoginRequiredMixin
from django.contrib import messages
from django.core.exceptions import ValidationError
from web.forms.project import ProjectCreateForm
import typing as t
from account.models import User
from flag.models import Project, ProjectAccess
from flag.models.access import Role


class ProjectCreateView(LoginRequiredMixin, View):
    """View for creating a new project"""

    def get(self, request: HttpRequest) -> HttpResponse:
        form = ProjectCreateForm()
        return render(
            request,
            "projects/create.html",
            {"form": form, "title": "Create New Project"},
        )

    def post(self, request: HttpRequest) -> HttpResponse:
        form = ProjectCreateForm(request.POST)

        if form.is_valid():
            project = form.save(commit=False)
            project.author = t.cast(User, request.user)
            project.save()

            # Create access
            # *Might wanna put this in a signal?
            ProjectAccess.objects.create(
                user=request.user,
                role=Role.ADMIN,
                project=project,
            )

            messages.success(request, f'Project "{project.name}" created successfully!')
            return redirect("home")

        return render(
            request,
            "projects/create.html",
            {"form": form, "title": "Create New Project"},
        )


def project_view(request: HttpRequest, id: str) -> HttpResponse:
    user = t.cast(User, request.user)

    try:
        project = Project.objects.filter(access__user__id=user.id).get(id=id)
    except Project.DoesNotExist:
        project = None
    except ValidationError:
        messages.error(request, "Invalid id")
        project = None

    return render(request, "project.html", {"project": project})
