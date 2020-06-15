# DinoPark Lookout (updating Dinos since 2019)
[![Build Status](https://travis-ci.org/mozilla-iam/dino-park-lookout.svg?branch=master)](https://travis-ci.org/mozilla-iam/dino-park-lookout)

DinoPark Lookout is the event listener of DinoPark.

It has two endpoints for web hooks:
- `/events/update` to trigger an individual profile update to search and orgchart (used by cis-notifier)
- `/bulk/update` and internal update to trigger updates for all profiles