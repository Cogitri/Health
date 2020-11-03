# Information that Health collects and stores

Health aims to provide a way to sync your step counts and weight measurement with Google Fit. For this to function, Health
downloads the data from the Google Fit API.

This data is stored locally on your computer and is not transmitted to the Health developers or any third parties. We do not serve any advertisements, including retargeting, personalized, or interest-based advertising.

Upon authenticating with your Google account, Health stores the access token it receives in your secure keyring (e.g. GNOME Keyring, KDE Wallet etc.).

Health will periodically use this access token for authenticating with Google when downloading updated data (e.g. when you've entered a new step count into the Google Fit Android App).


## Permissions Health requests

By default, Health requests the following scopes (permissions) with the Google API. Users are free to only allow part of the scope set, but that may reduce Health's functionality. The following scopes are used for the respective features:

* Access to Body Measurements (Read) - Required for importing weight measurements into Health.
* Access to Activities (Read) - Required for importing step counts into Health.
* Access to Body Measurements (Write) - Required for synching weight measurements to Google Fit upon entering them in Health.
* Access to Activities (Write) - Required for synching step counts to Google Fit upon entering them in Health.
