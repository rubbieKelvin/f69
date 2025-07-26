from django.shortcuts import render

def index(request):
    """Landing page for F69 feature flagging system"""
    return render(request, 'index.html')
