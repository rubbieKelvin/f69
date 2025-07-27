from django.views import View
from django.http import HttpRequest, HttpResponse
from django.shortcuts import render, redirect
from django.contrib.auth.mixins import LoginRequiredMixin
from django.contrib import messages
from django.core.exceptions import ValidationError
from web.forms.secret import SecretCreateForm
import typing as t
from account.models import User
from flag.models import (
    Project,
    ProjectClientSecret,
)
from flag.models.access import Role


def delete_secret_view(
    request: HttpRequest, project_id: str, secret_id: str
) -> HttpResponse:
    """View to delete/revoke an API secret"""
    user = t.cast(User, request.user)

    if request.method != "POST":
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
        messages.success(
            request, f'API secret "{secret_name}" has been revoked successfully.'
        )
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
