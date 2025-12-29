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

We want to reach out to users with legacy email providers. Flag users if their `email` contains `hotmail`.

* type: "CONTACT_CUSTOMER",
* reason: "LEGACY_EMAIL_PROVIDER"

### Team-2

American Express cards starting with `37` require additional verification due to higher processing fees. Flag credit cards that start with `37`.

* type: "CREDIT_CARD_VERIFICATION",
* reason: "AMEX_CARD"

### Team-3

Our sales team wants to target financial sector customers. Contact users if their `industry` contains `Bank`, `Financ`, `Insur`, `Invest`, `Account`, or `Capital`.

* type: "SALES_LEAD",
* reason: "FINANCIAL_SECTOR"

### Team-4

We are looking to acquire tech companies. Flag users if their `industry` contains `Computer`, `Internet`, `Semiconductor`, `Telecom`, or `Wireless`.

* type: "ACQUISITION_TARGET",
* reason: "TECH_COMPANY"

### Team-5

Due to compliance requirements, we cannot host healthcare companies. Contact users if their `industry` contains `Health`, `Hospital`, `Medical`, `Pharma`, `Biotech`, or `Veterinary`.

* type: "CONTACT_CUSTOMER",
* reason: "HEALTHCARE_COMPLIANCE"

### Team-6

We need more developers: We need to contact if the profession is an `engineer`.

* type: "HIRE_CUSTOMER",
* reason: "IS_ENGINEER"

### Team-7

We need to identify VIP users for special offers. Contact users if `premium` is `true` AND `credit` is greater than `10`.

* type: "CONTACT_CUSTOMER",
* reason: "VIP_USER"

### Team-8

Some customers have severely negative credits and need urgent attention. Warn users if `credit` is less than `-15`.

* type: "CONTACT_CUSTOMER",
* reason: "CRITICAL_DEBT"

### Team-9

We are expanding to Asia-Pacific markets. Contact users if their `time_zone` starts with `Asia/`.

* type: "CONTACT_CUSTOMER",
* reason: "APAC_EXPANSION"

### Team-10

Our frontend doesn't support Internet Explorer. Warn users if their `user_agent` contains `MSIE` or `Trident`.

* type: "CONTACT_CUSTOMER",
* reason: "LEGACY_BROWSER"

### Team-11

Due to GDPR regulations, we need to trigger a compliance workflow for users in Europe. Filter users if their `time_zone` starts with `Europe/`.

* type: "TRIGGER_GDPR_COMPLIANCE",
* reason: "IN_EUROPE"

### Team-12

We need tech professionals. Contact users if their `field` is `IT` or `Technology`.

* type: "HIRE_CUSTOMER",
* reason: "TECH_PROFESSIONAL"

### Team-13

We need to ensure profile completeness. Contact users if their `avatar` is `example.org` or doesn't start with `https://`.

* type: "CONTACT_CUSTOMER",
* reason: "INVALID_AVATAR"

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
