from django.contrib import admin
from .models import Entity, Feature, Flag, Project, Segment, ProjectAccess


@admin.register(Entity)
class EntityAdmin(admin.ModelAdmin): ...


@admin.register(Feature)
class FeatureAdmin(admin.ModelAdmin): ...


@admin.register(Flag)
class FlagAdmin(admin.ModelAdmin): ...


@admin.register(Project)
class ProjectAdmin(admin.ModelAdmin): ...


@admin.register(Segment)
class SegmentAdmin(admin.ModelAdmin): ...


@admin.register(ProjectAccess)
class ProjectAccessAdmin(admin.ModelAdmin): ...
