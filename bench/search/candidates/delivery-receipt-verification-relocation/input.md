# Product Delivery Runbook

## Intake

### Route Confirmation

- Confirm the driver assignment.
- Verify the delivery window with dispatch.

### Delivery Receipt Verification

Use this check when a carrier submits proof of delivery.

| Check | Evidence |
|---|---|
| Signature | Customer name and timestamp |
| Parcel count | Manifest total matches receipt |

```text
receipt_status: pending-review
required_fields: signature, parcel_count
```

- Escalate mismatches to the shift lead.
- Attach the receipt image to the ticket.

### Dispatch Notes

- Record traffic or weather delays.
- Keep the route summary in the trip log.

## Completion

### Customer Handoff

- Confirm the customer has the final receipt copy.
- Close the delivery ticket.

### Receipt Verification Exceptions

This subsection is a decoy and must stay under Completion.

- Damaged label exceptions are handled separately.
- Missing-signature exceptions require manager approval.

### Archive

> Old draft heading: ### Delivery Receipt Verification
> Keep archived wording as historical context.

### Return Intake

- Start a return ticket when items are refused.
- Note the reason supplied by the customer.

