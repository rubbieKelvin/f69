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
    path('accounts/signup/', auth.signup_view, name='signup'),
    path('accounts/login/', auth_views.LoginView.as_view(), name='login'),
    path('accounts/logout/', auth.logout_view, name='logout'),
]
