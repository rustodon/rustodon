# Stories

## Sooner
* 01) As an admin, I can block instances.
  - As a user, I cannot interact with blocked instances.

* 02) As a user, I can silence instances.
  - As a user, I cannot interact with users on a silenced instance unless I explicitly follow them.
  - As a user, I will not see statuses from users on a silenced instance unless I explicitly follow them.

* 03) As a user, I can block other users.
  - As a user, I will not see blocked users' statuses.
  - As a user, blocked users will not be able to interact with me.

* 04) As a user, I can mute other users.
  - As a user, I will not see muted users' posts.
  - As a user, I will be able to see notifications, with a toggle to hide them.

* 05) As a user, I can mute keywords.
  - As a user, I will not see posts with muted keywords.

* 06) As a user, I can create statuses.
  - As a user, I can control who sees my statuses by setting visibility.
    + As a user, I can hide statuses from the public timeline.
    + As a user, I can hide statuses from users who do not follow me.
    + As a user, I can hide statuses from everyone except those mentioned in the statuses.

* 07) As a user, my statuses are sent to other instances where I have followers.
  - As a user, my statuses don't get sent to blocked instances.
  - As a user, my statuses don't get sent to blocked users.

* 08) As a user, I can mention other users in statuses.
  - As a user, I will be notified if I am mentioned in a status.
  - As a remote ActivityPub user, I will be notified if I am mentioned in a status.
  - As a user, I will not see posts from users I follow that mention someone I do not follow.

* 09) As a user, I can reply to statuses.
  - As a user, I will not see replies from users I follow to users I do not follow.

* 10) As a remote ActivityPub user, I can retrieve public statuses from users.


* 11) As a user, I can follow other accounts and see their statuses in a timeline.
  - As a user, I can follow remote ActivityPub accounts and see their statuses.
  - As a user, I can approve or reject follow requests issued to my account.
  - As a user, I can issue a follow request to locked accounts.
    + As a user, I can rescind a follow request.

* 12) As a user, I can lock my account so that followers must be approved.
  - (C) in Later

* 13) As a user, I can see statuses of accounts I follow.
  - As a remote ActivityPub user, I can receive statuses from a user I follow.
    + As a remote ActivityPub user, I cannot receive statuses from a user who has blocked me.
  - As a local user, I can see statuses from a user I follow.
    + As a local user, I cannot see statuses from a user who has blocked me.
    + As a local user, I cannot see statuses from a user who I have blocked.

* 14) As a user, I can authenticate to the website.
  - As a user, I can register for an account on the website.

* 15) As a user, I can set a profile.
  - As a remote ActivityPub user, I can see these profiles.
  - (C) in Later

* 16) As a non-authenticated user, I can see user profiles on public pages.
  - As a non-authenticated user, I can see users' statuses on profile pages.


## Later
* 12c) As a user, I may specify accounts that may follow me without authentication, if I know I would like to be followed by them
  - I will still be notified when they do follow me.
  - a notification will show in the top of my notifications that may be dismissed until new follow requests arrive, alerting me of pending follow requests
  - A notification will be added to my notifications when a follow request I accepted completes, showing they did in fact follow me.
  
* 15c) As a user, I may opt out of public profile pages for non-authenticated users.
  - As a user, I may opt out/in of being indexed by search engines.
  
* 17) I can use an external client to interact with the application.
  - I can use my credentials to obtain a token for external interaction with the application.

* 18) As a user, I can report a status, instance, and/or user.
  - As a user, I can add statuses other than the one used to begin a report.
  - As an admin, I can view all reports and use a report to begin drafting other administrative actions.
  - Reports will federate to the instance that owns the user being reported, for user reports, and status reports
