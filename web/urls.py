from django.urls import path
from django.contrib.auth import views as auth_views
from . import views
from .views import auth
from .views import project

urlpatterns = [
    path('', views.index, name='index'),
    path('home/', auth.home_view, name='home'),
    path('projects/new/', project.ProjectCreateView.as_view(), name="create-project"),
    path('projects/<str:id>/', project.project_view, name="project-detail"),
    path('projects/<str:id>/features/', project.project_features_view, name="project-features"),
    path('projects/<str:id>/entities/', project.project_entities_view, name="project-entities"),
    path('projects/<str:id>/environments/', project.project_environments_view, name="project-environments"),
    path('projects/<str:id>/segments/', project.project_segments_view, name="project-segments"),
    path('projects/<str:id>/collaborators/', project.project_collaborators_view, name="project-collaborators"),
    path('accounts/signup/', auth.signup_view, name='signup'),
    path('accounts/login/', auth_views.LoginView.as_view(), name='login'),
    path('accounts/logout/', auth.logout_view, name='logout'),
]
