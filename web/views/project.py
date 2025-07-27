from django.views import View
from django.http import HttpRequest, HttpResponse
from django.shortcuts import render, redirect
from django.contrib.auth.mixins import LoginRequiredMixin
from django.contrib import messages
from django.conf import settings
from django.core.exceptions import ValidationError
from web.forms.project import ProjectCreateForm
from web.forms.feature import FeatureCreateForm
from web.forms.secret import SecretCreateForm
import typing as t
from account.models import User
from flag.models import (
    Project,
    ProjectAccess,
    Entity,
    Segment,
    Feature,
    ProjectClientSecret,
)
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
    """Project overview tab - main project page"""
    user = t.cast(User, request.user)

    try:
        project = Project.objects.filter(access__user__id=user.id).get(id=id)
    except Project.DoesNotExist:
        project = None
    except ValidationError:
        messages.error(request, "Invalid id")
        project = None

    if not project:
        return render(
            request,
            "project/overview.html",
            {"project": project, "current_tab": "overview"},
        )

    # Get overview data
    features = project.features.all()
    entities = project.entities.all()
    segments = project.segments.all()
    collaborators = project.access.all()

    context = {
        "project": project,
        "current_tab": "overview",
        "features_count": features.count(),
        "entities_count": entities.count(),
        "segments_count": segments.count(),
        "collaborators_count": collaborators.count(),
        "recent_features": features.order_by("-created_at")[:5],
    }

    return render(request, "project/overview.html", context)


def project_features_view(request: HttpRequest, id: str) -> HttpResponse:
    """Project features tab"""
    user = t.cast(User, request.user)

    try:
        project = Project.objects.filter(access__user__id=user.id).get(id=id)
    except Project.DoesNotExist:
        project = None
    except ValidationError:
        messages.error(request, "Invalid id")
        project = None

    if not project:
        return render(
            request,
            "project/features.html",
            {"project": project, "current_tab": "features"},
        )

    features = project.features.all().order_by("-created_at")

    context = {
        "project": project,
        "current_tab": "features",
        "features": features,
    }

    return render(request, "project/features.html", context)


def project_entities_view(request: HttpRequest, id: str) -> HttpResponse:
    """Project entities tab"""
    user = t.cast(User, request.user)

    try:
        project = Project.objects.filter(access__user__id=user.id).get(id=id)
    except Project.DoesNotExist:
        project = None
    except ValidationError:
        messages.error(request, "Invalid id")
        project = None

    if not project:
        return render(
            request,
            "project/entities.html",
            {"project": project, "current_tab": "entities"},
        )

    entities = project.entities.all().order_by("-created_at")
    entity_tags = entities.values_list("tag", flat=True).distinct()

    context = {
        "project": project,
        "current_tab": "entities",
        "entities": entities,
        "entity_tags": entity_tags,
    }

    return render(request, "project/entities.html", context)


def project_environments_view(request: HttpRequest, id: str) -> HttpResponse:
    """Project environments tab"""
    user = t.cast(User, request.user)

    try:
        project = Project.objects.filter(access__user__id=user.id).get(id=id)
    except Project.DoesNotExist:
        project = None
    except ValidationError:
        messages.error(request, "Invalid id")
        project = None

    if not project:
        return render(
            request,
            "project/environments.html",
            {"project": project, "current_tab": "environments"},
        )

    # For now, environments will be empty since there's no environment model yet
    # This can be updated when the environment model is created
    environments = []

    context = {
        "project": project,
        "current_tab": "environments",
        "environments": environments,
    }

    return render(request, "project/environments.html", context)


def project_segments_view(request: HttpRequest, id: str) -> HttpResponse:
    user = t.cast(User, request.user)

    try:
        project = Project.objects.filter(access__user__id=user.id).get(id=id)
    except Project.DoesNotExist:
        project = None
    except ValidationError:
        messages.error(request, "Invalid id")
        project = None

    if not project:
        return render(
            request,
            "project/segments.html",
            {"project": project, "current_tab": "segments"},
        )

    segments = project.segments.all().order_by("-created_at")

    context = {
        "project": project,
        "current_tab": "segments",
        "segments": segments,
    }

    return render(request, "project/segments.html", context)


def project_collaborators_view(request: HttpRequest, id: str) -> HttpResponse:
    user = t.cast(User, request.user)

    try:
        project = Project.objects.filter(access__user__id=user.id).get(id=id)
    except Project.DoesNotExist:
        project = None
    except ValidationError:
        messages.error(request, "Invalid id")
        project = None

    if not project:
        return render(
            request,
            "project/collaborators.html",
            {"project": project, "current_tab": "collaborators"},
        )

    collaborators = project.access.select_related("user").all().order_by("-created_at")

    context = {
        "project": project,
        "current_tab": "collaborators",
        "collaborators": collaborators,
    }

    return render(request, "project/collaborators.html", context)


def project_api_view(request: HttpRequest, id: str) -> HttpResponse:
    api_url = "http://127.0.0.1:8000/f/flags" if settings.DEBUG else "https://api.f69.dev/f/flags"
    user = t.cast(User, request.user)

    try:
        project = Project.objects.filter(access__user__id=user.id).get(id=id)
    except Project.DoesNotExist:
        project = None
    except ValidationError:
        messages.error(request, "Invalid id")
        project = None

    if not project:
        return render(
            request,
            "project/api.html",
            {
                "project": project,
                "current_tab": "api",
                "api_url": api_url,
            },
        )

    # Get project secrets
    secrets = ProjectClientSecret.objects.filter(project=project).order_by(
        "-created_at"
    )

    context = {
        "client_id": user.id,
        "project": project,
        "current_tab": "api",
        "secrets": secrets,
        "api_url": api_url
    }

    return render(request, "project/api.html", context)


def delete_secret_view(request: HttpRequest, project_id: str, secret_id: str) -> HttpResponse:
    """View to delete/revoke an API secret"""
    user = t.cast(User, request.user)
    
    if request.method != 'POST':
        messages.error(request, "Invalid request method")
        return redirect("project-api", id=project_id)
    
    # Get the project and verify access
    try:
        project = Project.objects.filter(access__user__id=user.id).get(id=project_id)
    except Project.DoesNotExist:
        messages.error(request, "Project not found or access denied")
        return redirect("home")
    except ValidationError:
        messages.error(request, "Invalid project ID")
        return redirect("home")
    
    # Get the secret and verify it belongs to this project
    try:
        secret = ProjectClientSecret.objects.get(id=secret_id, project=project)
        secret_name = secret.name
        secret.delete()
        messages.success(request, f'API secret "{secret_name}" has been revoked successfully.')
    except ProjectClientSecret.DoesNotExist:
        messages.error(request, "Secret not found or access denied")
    except ValidationError:
        messages.error(request, "Invalid secret ID")
    except Exception as e:
        messages.error(request, "An error occurred while revoking the secret")
    
    return redirect("project-api", id=project_id)


class SecretCreateView(LoginRequiredMixin, View):
    """View for creating a new API secret"""

    def get(self, request: HttpRequest, project_id: str) -> HttpResponse:
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

        form = SecretCreateForm()
        return render(
            request,
            "project/create-secret.html",
            {
                "form": form,
                "project": project,
                "title": f"Create API Secret - {project.name}",
            },
        )

    def post(self, request: HttpRequest, project_id: str) -> HttpResponse:
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

        form = SecretCreateForm(request.POST)

        if form.is_valid():
            try:
                # Use the static method to create the secret
                plain_secret, secret_obj = ProjectClientSecret.new(
                    name=form.cleaned_data["name"], author=user, project=project
                )

                # Store the plain secret in session to show once
                request.session["new_secret"] = {
                    "value": plain_secret,
                    "name": secret_obj.name,
                    "id": str(secret_obj.id),
                }

                messages.success(
                    request, f'API secret "{secret_obj.name}" created successfully!'
                )
                return redirect("secret-created", project_id=project.id)

            except Exception as e:
                messages.error(request, "An error occurred while creating the secret.")

        return render(
            request,
            "project/create-secret.html",
            {
                "form": form,
                "project": project,
                "title": f"Create API Secret - {project.name}",
            },
        )


def secret_created_view(request: HttpRequest, project_id: str) -> HttpResponse:
    """View to display the newly created secret (one-time only)"""
    user = t.cast(User, request.user)

    # Get the project and verify access
    try:
        project = Project.objects.filter(access__user__id=user.id).get(id=project_id)
    except Project.DoesNotExist:
        messages.error(request, "Project not found or access denied")
        return redirect("home")
    except ValidationError:
        messages.error(request, "Invalid project ID")
        return redirect("home")

    # Get the secret from session (one-time display)
    secret_data = request.session.pop("new_secret", None)

    if not secret_data:
        messages.warning(request, "Secret information is no longer available.")
        return redirect("project-api", id=project.id)

    return render(
        request,
        "project/secret-created.html",
        {
            "project": project,
            "secret_data": secret_data,
            "title": f"Secret Created - {project.name}",
            "client_id": user.id,
        },
    )


class FeatureCreateView(LoginRequiredMixin, View):
    def get(self, request: HttpRequest, project_id: str) -> HttpResponse:
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

        form = FeatureCreateForm()
        return render(
            request,
            "project/create-feature.html",
            {
                "form": form,
                "project": project,
                "title": f"Create Feature Flag - {project.name}",
            },
        )

    def post(self, request: HttpRequest, project_id: str) -> HttpResponse:
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

        form = FeatureCreateForm(request.POST)

        if form.is_valid():
            feature = form.save(commit=False)
            feature.project = project
            feature.author = user

            try:
                feature.save()
                messages.success(
                    request, f'Feature "{feature.name}" created successfully!'
                )
                return redirect("project-features", id=project.id)
            except Exception as e:
                # Handle unique constraint errors or other database issues
                if "unique" in str(e).lower():
                    form.add_error(
                        "slug",
                        "A feature with this slug already exists in this project.",
                    )
                else:
                    messages.error(
                        request, "An error occurred while creating the feature."
                    )

        return render(
            request,
            "project/create-feature.html",
            {
                "form": form,
                "project": project,
                "title": f"Create Feature Flag - {project.name}",
            },
        )
