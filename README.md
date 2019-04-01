# DinoPark Lookout (updating Dinos since 2019)
[![Build Status](https://travis-ci.org/mozilla-iam/dino-park-lookout.svg?branch=master)](https://travis-ci.org/mozilla-iam/dino-park-lookout)
![Build Status](https://codebuild.us-west-2.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoidDhleWxKUkVYNjRNZ2xiWDRyMGZ0RkJBdS9MOEtrTVgxN29VcmdEMVpWcVgzcXlxc0Zmc0pGRzA5YW9COC9wMWovQ0ZRNlFBV25TT1JzNHRaaDdHckNrPSIsIml2UGFyYW1ldGVyU3BlYyI6ImE1K3RpK2lmWFBJZnF3Nk8iLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=master)

DinoPark Lookout is the event listener of DinoPark.

It has two endpoints for web hooks:
- `/events/update` to trigger an individual profile update to search and orgchart (used by cis-notifier)
- `/bulk/update` and internal update to trigger updates for all profiles