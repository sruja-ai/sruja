context auth in identity { owner: team.identity }
context patientCore in patients { owner: team.patients }
context schedule in appointments { owner: team.appt }
context emr in records { owner: team.records }
context claims in billing { owner: team.billing }
context results in labs { owner: team.labs }