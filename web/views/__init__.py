from django.shortcuts import render, redirect
from django.http import HttpRequest, HttpResponse


# Index
def index(request: HttpRequest) -> HttpResponse:
    if request.user.is_authenticated:
        return redirect("home")

    context = {"name": "F69"}
    return render(request, "index.html", context)
