Compact app button for the settings window — header actions, row buttons, save bar, side-nav tabs.
```jsx
<Button variant="primary" onClick={openChat}>Open chat</Button>
<Button>Restart</Button>
<Button variant="danger">Exit</Button>
<Button variant="tab" active>Overview</Button>
```
- One primary per view (Open chat, Save). Labels are sentence case verbs: "Check for updates", "Open models folder".
- Disabled = 50% opacity, no hover.
