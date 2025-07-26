import typing as t
from django.shortcuts import render, redirect
from django.contrib.auth import login, logout
from django.contrib.auth.decorators import login_required
from django.contrib import messages
from django import forms
from account.models import User
from django.http import HttpRequest, HttpResponse
from django.conf import settings
from flag.models import Project
from account.models import User
from web.forms.auth import UserCreationForm

# Signup
def signup_view(request: HttpRequest) -> HttpResponse:
    if request.user.is_authenticated:
        return redirect("home")

    if request.method == "POST":
        form = UserCreationForm(request.POST)
        if form.is_valid():
            user = form.save()
            email = form.cleaned_data.get("email")

            # Flash message
            messages.success(request, f"Account created for {email}!")

            # Login user, then go home
            login(request, user)
            return redirect("home")
    else:
        form = UserCreationForm()

    return render(request, "registration/signup.html", {"form": form})


@login_required
def home_view(request: HttpRequest) -> HttpResponse:
    user = t.cast(User, request.user)
    projects = Project.objects.filter(is_deleted=False, author__id=user.id)

    return render(request, "home.html", {"user": request.user, "projects": projects})


@login_required
def logout_view(request: HttpRequest) -> HttpResponse:
    logout(request)
    return redirect(settings.LOGOUT_REDIRECT_URL)
