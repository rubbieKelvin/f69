from django.urls import path
from django.contrib.auth import views as auth_views
from . import views
from .views import auth
from .views import project
from .views import secrets
from .views import features

urlpatterns = [
    path('', views.index, name='index'),
    path('home/', auth.home_view, name='home'),
    path('projects/new/', project.ProjectCreateView.as_view(), name="create-project"),
    path('projects/<str:id>/', project.project_view, name="project-detail"),
    path('projects/<str:id>/features/', project.project_features_view, name="project-features"),
    path('projects/<str:project_id>/features/<str:feature_id>/', features.feature_detail_view, name="feature-detail"),
    path('projects/<str:project_id>/features/new/', project.FeatureCreateView.as_view(), name="create-feature"),
    path('projects/<str:project_id>/secrets/new/', secrets.SecretCreateView.as_view(), name="create-secret"),
    path('projects/<str:project_id>/secrets/created/', secrets.secret_created_view, name="secret-created"),
    path('projects/<str:project_id>/secrets/<str:secret_id>/delete/', secrets.delete_secret_view, name="delete-secret"),
    path('projects/<str:project_id>/environments/new/', project.environment_create_view, name="create-environment"),
    path('projects/<str:project_id>/segments/new/', project.SegmentCreateView.as_view(), name="create-segment"),
    path('projects/<str:project_id>/segments/<str:segment_id>/edit/', project.SegmentEditView.as_view(), name="edit-segment"),
    path('projects/<str:project_id>/entities/<str:entity_id>/delete/', project.delete_entity_view, name="delete-entity"),
    path('projects/<str:id>/entities/', project.project_entities_view, name="project-entities"),
    path('projects/<str:id>/environments/', project.project_environments_view, name="project-environments"),
    path('projects/<str:id>/segments/', project.project_segments_view, name="project-segments"),
    path('projects/<str:id>/collaborators/', project.project_collaborators_view, name="project-collaborators"),
    path('projects/<str:id>/api/', project.project_api_view, name="project-api"),
    path('accounts/signup/', auth.signup_view, name='signup'),
    path('accounts/login/', auth_views.LoginView.as_view(), name='login'),
    path('accounts/logout/', auth.logout_view, name='logout'),
]
