# Stories

## Sooner
* As an admin, I can block instances.
  - As a user, I cannot interact with blocked instances.

* As a user, I can silence instances.
  - As a user, I cannot interact with users on a silenced instance unless I explicitly follow them.
  - As a user, I will not see statuses from users on a silenced instance unless I explicitly follow them.

* As a user, I can block other users.
  - As a user, I will not see blocked users' statuses.


* As a user, I can create statuses.
  - As a user, I can control who sees my statuses by setting visibility.
    + As a user, I can hide statuses from the public timeline.
    + As a user, I can hide statuses from users who do not follow me.
    + As a user, I can hide statuses from everyone except those mentioned in the statuses.

* As a user, my statuses are sent to other instances where I have followers.
  - As a user, my statuses don't get sent to blocked instances.
  - As a user, my statuses don't get sent to blocked users.

* As a user, I can mention other users in statuses.
  - As a user, I will be notified if I am mentioned in a status.
  - As a remote ActivityPub user, I will be notified if I am mentioned in a status.

* As a remote ActivityPub user, I can retrieve public statuses from users.


* As a user, I can follow other accounts and see their statuses in a timeline.
  - As a user, I can follow remote ActivityPub accounts and see their statuses.
  - As a user, I can issue a follow request to locked accounts.

* As a user, I can lock my account so that followers must be approved.
  - As a user, I can approve or reject follow requests issued to my account.

* As a user, I can see statuses of accounts I follow.
  - As a remote ActivityPub user, I can receive statuses from a user I follow.
    + As a remote ActivityPub user, I cannot receive statuses from a user who has blocked me.
  - As a local user, I can see statuses from a user I follow.
    + As a local user, I cannot see statuses from a user who has blocked me.
    + As a local user, I cannot see statuses from a user who I have blocked.


* As a user, I can authenticate to the website
  - As a user, I can register for an account on the website.

* As a user, I can set a profile.
  - As a remote ActivityPub user, I can see these profiles.


* As a non-authenticated user, I can see user profiles on public pages.
  - As a non-authenticated user, I can see users' statuses on profile pages.

## Later
* I can use an external client to interact with the application.
  - I can use my credentials to obtain a token for external interaction with the application.