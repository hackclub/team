# team
### HQ'ers, the [Airtable with your info is here](https://airtable.com/invite/l?inviteId=invgnVcC5Hrs5O9VZ&inviteToken=0341506c21006470a934631546c84dcbe42ac52d3093b7f4b4ca51eaef654a82&utm_medium=email&utm_source=product_team&utm_content=transactional-alerts).
It's historically been tricky for less code-inclined Hack Club staff members to update their bios etc. on hackclub.com/team.

As I'm building out The Hack Foundation's new website, and need a team page for that as well, I thought it would be worthwile to have one source of truth for teams' descriptions.

The Airtable base automatically updates this API with the following script, which is hooked into record edits;

```js
const currentEmployeeTable = base.getTable("Current");

const fields = Object.values(currentEmployeeTable.fields).map((r) => r.name);

const currentEmployeeRecords = await currentEmployeeTable.selectRecordsAsync({ fields });

const currentEmployees = currentEmployeeRecords.records
    .map((record) => {
        const final = {};
        for (const field of currentEmployeeTable.fields) {
			if (field.type === "singleSelect") {
                final[field.name] = record.getCellValueAsString(field);
            } else {
                final[field.name] = record.getCellValue(field);
            }
        }
        return final;
    });

const teamServerSecret = input.config()["team server secret"];
const url = `https://internal.bank.engineering/team/?token=${teamServerSecret}`;
const response = await fetch(url, {
    method: "POST",
    body: JSON.stringify({
        current: currentEmployees,
        alumni: [] //TODO?
    }),
    headers: { "Content-Type": "application/json" },
});

await response.json().then(console.log)
```
