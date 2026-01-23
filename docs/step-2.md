# Step 2: Transform

Now that you’ve parsed your JSON message into a structured format, it’s time to perform some verifications on the user data based on your team’s specific task.

## TODOs

1. **Filter Users:** Implement filters based on your team's assigned criteria. Each team has a unique objective to achieve, which can be seen below.
2. **Assign Output Labels:** When a user matches your filter, you'll produce an action message in Step 3. This message needs two labels from your team's row in the table below:
   - `type`: The category of action (e.g., `CONTACT_CUSTOMER`, `SALES_LEAD`)
   - `reason`: Why this user was flagged (e.g., `LEGACY_EMAIL_PROVIDER`, `VIP_USER`)

These are **NOT** fields in the incoming user data—they're predetermined values you ADD to your output.

### Understanding the Data Flow

```
INPUT (from new_users topic)          OUTPUT (to actions topic)
┌─────────────────────────────┐       ┌────────────────────────────────────┐
│ {                           │       │ {                                  │
│   "email": "bob@hotmail.com"│       │   "customer": "bob@hotmail.com",   │
│   "company": "Acme Inc",    │  ──►  │   "type": "CONTACT_CUSTOMER",      │ ← From your team's row
│   "premium": true,          │       │   "reason": "LEGACY_EMAIL_PROVIDER"│ ← From your team's row
│   "credit": 15,             │       │   "team": "team-1"                 │
│   ...                       │       │ }                                  │
│ }                           │       └────────────────────────────────────┘
└─────────────────────────────┘
     ↑ No type/reason here!              ↑ You ADD type/reason here!
```

**Key insight:** The `type` and `reason` values come from YOUR TEAM'S ROW in the table below, NOT from the user data.

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

## Handling Missing or Invalid Data

Real-world data is messy. While our tutorial producer sends well-formed JSON, production systems should always handle potential errors:

```python
for message in consumer:
    try:
        user = json.loads(message.value.decode('utf-8'))

        # Use .get() with defaults for safer field access
        industry = user.get('industry', '').lower()  # Returns '' if field missing

        if 'tech' in industry:
            process_user(user)

    except json.JSONDecodeError:
        print(f"Invalid JSON: {message.value}")
    except KeyError as e:
        print(f"Missing required field: {e}")
```

### Key Patterns

| Pattern | Use Case |
|---------|----------|
| `user.get('field', '')` | Optional fields - returns default if missing |
| `user['field']` | Required fields - raises KeyError if missing |
| `try/except` | Catches malformed messages without crashing |

> **Note:** For this tutorial, the data is always valid. But building these habits early will save you debugging time in production!

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

## Check Your Work

Your filter logic runs locally - you'll verify it works in the next step when you produce messages. If 📤 appears on the leaderboard, your filter is matching users correctly!

---

## Next step

Congratulations, you learned how to apply a filter on a Kafka stream 🎉
You can now head to [step 3](/kafka-tutorial/docs/step-3.html)!
