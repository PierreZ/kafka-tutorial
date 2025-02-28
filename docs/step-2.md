# Step 2: Transform

Now that you’ve parsed your JSON message into a structured format, it’s time to perform some verifications on the user data based on your team’s specific task.

## TODOs

1. **Filter Users:** Implement filters based on your team’s assigned criteria. Each team has a unique objective to achieve, chich can be seen below.
2. **Retain Key Fields:** Ensure your processing includes the following fields:
   - `type`: Describes the operation being performed.
   - `reason`: Explains the context or motivation for the operation.

These fields are crucial for generating the next Kafka message, so be sure to include them in your output.

## Team's criteria

### Team-1

We are receiving a lot of spams from Yahoo. We need to send a verification mail if the `email` field is from yahoo.

* type: "EMAIL_VERIFICATION",
* reason: "MIGHT_BE_FRAUD"

### Team-2

We have a lot of stolen credit-card. We need to check credit-card if it is starting with `55`.

* type: "CREDIT_CARD_VERIFICATION",
* reason: "MIGHT_BE_FRAUD"

### Team-3

We need to boost sales. We need to contact the customer if the `company_name` contains `Inc`.

* type: "CONTACT_CUSTOMER",
* reason: "UPSALE"

### Team-4

We need to buy other companies. We need to contact them if the `company_slogan` contains `middleware`.

* type: "CONTACT_CUSTOMER",
* reason: "BUY"

### Team-5

We cannot host medical companies. We need to contact if the industry contains words like `Hospital`, `Health` or`Care`

* type: "CONTACT_CUSTOMER",
* reason: "BAN"

### Team-6

We need more developers: We need to contact if the profession is an `engineer`.

* type: "HIRE_CUSTOMER",
* reason: "IS_ENGINEER"

### Team-7

We need to take care of premium users. We need to contact them if the `premium` is `true`.

* type: "CONTACT_CUSTOMER",
* reason: "PREMIUM"

### Team-8

Some customers have negative credits, we need to warn them.

* type: "CONTACT_CUSTOMER",
* reason: "LOW_CREDIT"

### Team-9

Due to some technical issues, we need to trigger some action if the user is not in Europe.

* type: "TRIGGER_JOB_BILLING",
* reason: "NOT_IN_EUROPE"

### Team-10

Due to the frontend stack, we need to warn users if there `user_agent` contains `Windows NT`.

* type: "CONTACT_CUSTOMER",
* reason: "UNSUPPORTED_WINDOWS_PLATFORM"

### Team-11

We need to provide analytics according to the time_zone. Read 100 messages, and count how many messages are from Europe. 

* type: "ANALYTICS_TIME_ZONE_EUROPE",
* reason: "your count as a string"

### Team-12

We need more HRs: We need to contact if the profession is an `human resources`.

* type: "HIRE_CUSTOMER",
* reason: "IS_HR"

### Team-13

We need to check the avatars: we need to contact if the avatar is not from robothash.

* type: "CONTACT_CUSTOMER",
* reason: "BAD_AVATAR"

### Team-14

We need to check the name: we need to ban the user if it's name is "John Doe".

* type: "BAN_CUSTOMER",
* reason: "SUSPICIOUS_NAME"

### Team-15

We need to check the pack, we need to send a mail to all `free` pack.

* type: "CONTACT_CUSTOMER",
* reason: "UPGRADE_FREE"
  
## Next step

Congratulations, you learned how to apply a filter on a Kafka stream 🎉
You can now head to [step 3](/kafka-tutorial/docs/step-3.html)!
