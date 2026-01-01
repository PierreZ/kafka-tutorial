# Step 2: Transform

Now that you’ve parsed your JSON message into a structured format, it’s time to perform some verifications on the user data based on your team’s specific task.

## TODOs

1. **Filter Users:** Implement filters based on your team's assigned criteria. Each team has a unique objective to achieve, which can be seen below.
2. **Retain Key Fields:** Ensure your processing includes the following fields:
   - `type`: Describes the operation being performed.
   - `reason`: Explains the context or motivation for the operation.

These fields are crucial for generating the next Kafka message, so be sure to include them in your output.

## Filter Examples

Here are some common patterns you'll need:

### String contains (case-insensitive)
```python
# Team-1 style: check if email contains "hotmail"
if "hotmail" in user["email"].lower():
    # matches!
```

### String starts with
```python
# Team-9 style: check if time_zone starts with "Asia/"
if user["time_zone"].startswith("Asia/"):
    # matches!
```

### Multiple conditions
```python
# Team-7 style: premium AND credit > 10
if user["premium"] == True and user["credit"] > 10:
    # matches!
```

### Check multiple values
```python
# Team-3 style: industry contains any of these keywords
keywords = ["Bank", "Financ", "Insur", "Invest", "Account", "Capital"]
if any(kw in user["industry"] for kw in keywords):
    # matches!
```

---

## Team's Criteria

| Team | Field | Condition | type | reason |
|------|-------|-----------|------|--------|
| 1 | `email` | contains `hotmail` | CONTACT_CUSTOMER | LEGACY_EMAIL_PROVIDER |
| 2 | `credit_card_number` | starts with `37` | CREDIT_CARD_VERIFICATION | AMEX_CARD |
| 3 | `industry` | contains `Bank`, `Financ`, `Insur`, `Invest`, `Account`, or `Capital` | SALES_LEAD | FINANCIAL_SECTOR |
| 4 | `industry` | contains `Computer`, `Internet`, `Semiconductor`, `Telecom`, or `Wireless` | ACQUISITION_TARGET | TECH_COMPANY |
| 5 | `industry` | contains `Health`, `Hospital`, `Medical`, `Pharma`, `Biotech`, or `Veterinary` | CONTACT_CUSTOMER | HEALTHCARE_COMPLIANCE |
| 6 | `profession` | is `engineer` | HIRE_CUSTOMER | IS_ENGINEER |
| 7 | `premium` + `credit` | `premium` = true AND `credit` > 10 | CONTACT_CUSTOMER | VIP_USER |
| 8 | `credit` | < -15 | CONTACT_CUSTOMER | CRITICAL_DEBT |
| 9 | `time_zone` | starts with `Asia/` | CONTACT_CUSTOMER | APAC_EXPANSION |
| 10 | `user_agent` | contains `MSIE` or `Trident` | CONTACT_CUSTOMER | LEGACY_BROWSER |
| 11 | `time_zone` | starts with `Europe/` | TRIGGER_GDPR_COMPLIANCE | IN_EUROPE |
| 12 | `field` | is `IT` or `Technology` | HIRE_CUSTOMER | TECH_PROFESSIONAL |
| 13 | `avatar` | contains `example.org` OR doesn't start with `https://` | CONTACT_CUSTOMER | INVALID_AVATAR |
| 14 | `name` | is `John Doe` | BAN_CUSTOMER | SUSPICIOUS_NAME |
| 15 | `pack` | is `free` | CONTACT_CUSTOMER | UPGRADE_FREE |

---

## Achievements

When you produce actions in the next step, watch the leaderboard for feedback:
- **False Positive**: You flagged a user who doesn't match your filter criteria
- **Ghost User**: The customer email doesn't exist in the `new_users` topic

Getting your filter logic right is key to earning points!

---

## Next step

Congratulations, you learned how to apply a filter on a Kafka stream 🎉
You can now head to [step 3](/kafka-tutorial/docs/step-3.html)!
