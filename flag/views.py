from django.shortcuts import render
from rest_framework.decorators import api_view, renderer_classes
from rest_framework.request import Request
from rest_framework.response import Response
from rest_framework.renderers import JSONRenderer

# for auth here, we'll need
# client_id
# project_id
# project_secret

@api_view(["post"])
@renderer_classes([])
def cook_feature_flags(request: Request, project_id: str) -> Response:
    return Response([])
