from django.views import View
from django.http import HttpRequest, HttpResponse
from django.shortcuts import render, redirect
from django.contrib.auth.mixins import LoginRequiredMixin
from django.contrib import messages
from django.conf import settings
from django.core.exceptions import ValidationError
from web.forms.project import ProjectCreateForm
from web.forms.feature import FeatureCreateForm
from web.forms.segment import SegmentCreateForm, SegmentEditForm
import typing as t
from account.models import User
from flag.models import (
    Project,
    ProjectAccess,
    Entity,
    Segment,
    Feature,
    ProjectClientSecret,
    Environment,
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

            # Create environment
            Environment.objects.create(
                name="Production",
                project=project,
            )

            # Create environment
            Environment.objects.create(
                name="Staging",
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

    # Get project environments
    environments = project.environments.all().order_by("-created_at")

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
    api_url = (
        "http://127.0.0.1:8000/f/flags"
        if settings.DEBUG
        else "https://api.f69.dev/f/flags"
    )
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
        "api_url": api_url,
    }

    return render(request, "project/api.html", context)


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


def environment_create_view(request: HttpRequest, project_id: str) -> HttpResponse:
    """Placeholder view for creating environments - to be implemented"""
    messages.info(request, "Environment creation is coming soon!")
    return redirect("project-environments", id=project_id)


class SegmentCreateView(LoginRequiredMixin, View):
    """View for creating segments"""

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

        form = SegmentCreateForm()

        return render(
            request,
            "project/create-segment.html",
            {
                "form": form,
                "project": project,
                "title": f"Create Segment - {project.name}",
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

        form = SegmentCreateForm(request.POST)

        if form.is_valid():
            try:
                segment = form.save(commit=False)
                segment.project = project
                segment.save()

                messages.success(
                    request, f'Segment "{segment.name}" created successfully!'
                )
                return redirect("project-segments", id=project_id)
            except Exception as e:
                messages.error(request, "An error occurred while creating the segment.")

        return render(
            request,
            "project/create-segment.html",
            {
                "form": form,
                "project": project,
                "title": f"Create Segment - {project.name}",
            },
        )


class SegmentEditView(LoginRequiredMixin, View):
    """View for editing segments and managing entities"""

    def get(
        self, request: HttpRequest, project_id: str, segment_id: str
    ) -> HttpResponse:
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

        # Get the segment and verify it belongs to this project
        try:
            segment = project.segments.get(id=segment_id)
        except Segment.DoesNotExist:
            messages.error(request, "Segment not found")
            return redirect("project-segments", id=project_id)
        except ValidationError:
            messages.error(request, "Invalid segment ID")
            return redirect("project-segments", id=project_id)

        form = SegmentEditForm(instance=segment, project=project)

        return render(
            request,
            "project/edit-segment.html",
            {
                "form": form,
                "project": project,
                "segment": segment,
                "title": f"Edit Segment - {segment.name}",
            },
        )

    def post(
        self, request: HttpRequest, project_id: str, segment_id: str
    ) -> HttpResponse:
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

        # Get the segment and verify it belongs to this project
        try:
            segment = project.segments.get(id=segment_id)
        except Segment.DoesNotExist:
            messages.error(request, "Segment not found")
            return redirect("project-segments", id=project_id)
        except ValidationError:
            messages.error(request, "Invalid segment ID")
            return redirect("project-segments", id=project_id)

        form = SegmentEditForm(request.POST, instance=segment, project=project)

        if form.is_valid():
            try:
                segment = form.save()
                messages.success(
                    request, f'Segment "{segment.name}" updated successfully!'
                )
                return redirect("project-segments", id=project_id)
            except Exception as e:
                messages.error(request, "An error occurred while updating the segment.")

        return render(
            request,
            "project/edit-segment.html",
            {
                "form": form,
                "project": project,
                "segment": segment,
                "title": f"Edit Segment - {segment.name}",
            },
        )


def delete_entity_view(
    request: HttpRequest, project_id: str, entity_id: str
) -> HttpResponse:
    """View to delete an entity"""
    user = t.cast(User, request.user)

    if request.method != "POST":
        messages.error(request, "Invalid request method")
        return redirect("project-entities", id=project_id)

    # Get the project and verify access
    try:
        project = Project.objects.filter(access__user__id=user.id).get(id=project_id)
    except Project.DoesNotExist:
        messages.error(request, "Project not found or access denied")
        return redirect("home")
    except ValidationError:
        messages.error(request, "Invalid project ID")
        return redirect("home")

    # Get the entity and verify it belongs to this project
    try:
        entity = project.entities.get(id=entity_id)
        entity_name = entity.name
        entity.delete()
        messages.success(
            request, f'Entity "{entity_name}" has been deleted successfully.'
        )
    except Entity.DoesNotExist:
        messages.error(request, "Entity not found or access denied")
    except ValidationError:
        messages.error(request, "Invalid entity ID")
    except Exception as e:
        messages.error(request, "An error occurred while deleting the entity")

    return redirect("project-entities", id=project_id)
