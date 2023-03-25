# twitter-collection-bot

A twitter bot for continuously posting a collection of media at exact timestamps

## Collection

The collection should be an sqlite3 database with *only* the following rows in order

| Purpose | Type | Notes |
| ------- | ---- | ----- |
| Tweet ID | `u64` | |
| File Names | `String` | comma-separated |
| Tweet Text Content | `String` | |
| Tweet Translated Text | `String` | |
| Tweet Date | `u64` | unix timestamp |

Set `COLLECTION_PATH` to the path of the sqlite3 database

Set `IMAGE_DIR_PATH` to the path of the image directory (where file names are contained)

## Twitter Auth

### Developer Bot

Provide the following environment variables

```bash
TWITTER_CONSUMER_KEY= # api key
TWITTER_CONSUMER_SECRET= # api secret
TWITTER_ACCESS_KEY= # access token
TWITTER_ACCESS_SECRET= # access secret
```

### *Experimental* User Bot

Provide the following environment variables

```bash
# Set this env variable to use a user account
TWITTER_USE_USER=1

TWITTER_USER_TOKEN= # authorization header
TWITTER_USER_AUTH_TOKEN= # auth_token cookie
```
