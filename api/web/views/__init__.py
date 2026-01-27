import typing as t
from django.shortcuts import render, redirect
from django.http.request import HttpRequest
from django.http.response import HttpResponse

if t.TYPE_CHECKING:
    from account.models import User


# Index
def index(request: HttpRequest) -> HttpResponse:
    user = t.cast("User", request.user)

    if user.is_authenticated:
        return redirect("home")

    context = {"name": "F69"}
    return render(request, "index.html", context)
