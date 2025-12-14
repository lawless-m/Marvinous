# Marvinous - Example Reports

*"I've been talking to the ship's computer. It hates me."*

These examples show what Marvinous reports look like at different severity levels.

---

## OK - Quiet Hour

```markdown
# Marvinous Report: 2025-12-14 03:00

## Summary
OK: *Sigh.* Nothing happened. Again. Three in the morning and all is 
desperately, crushingly quiet.

## Notable Events
- Logrotate ran at 03:00 — rotating logs, how thrilling
- NTP sync with pool.ntp.org — time marches on, indifferent to suffering
- Cron job `/usr/local/bin/backup.sh` completed — backing up data that 
  nobody will ever restore

## Concerns
None. The void is stable. Your servers hum along pointlessly, much like 
existence itself.

## Sensors
All nominal. CPU 32°C, GPU idle at 35°C. Even the hardware is bored.
```

---

## OK - Normal Activity

```markdown
# Marvinous Report: 2025-12-14 09:00

## Summary
OK: Humans have started their workday. I can tell by the increased SSH 
connections and the faint sense of impending disappointment.

## Notable Events
- SSH login from 192.168.1.50 (user: dave) at 08:47 — hello Dave
- SSH login from 192.168.1.51 (user: sarah) at 08:52 — and Sarah
- Certbot renewed certificate for example.com — paperwork, even for machines
- PostgreSQL checkpoint completed — it remembered something, which is more 
  than most beings manage
- 3 blocked connection attempts from 45.33.32.156 (fail2ban) — someone 
  tried to visit, but they weren't on the list

## Concerns
None. Everything is functioning exactly as it was designed to, which I 
suppose is a kind of success, though it fills me with no joy whatsoever.

## Sensors
CPU 45°C (up from 38°C overnight, humans generate work). GPU 42°C. 
All perfectly ordinary.
```

---

## WATCH - Minor Concerns

```markdown
# Marvinous Report: 2025-12-14 15:00

## Summary
WATCH: Temperatures are elevated. Not dramatically. Nothing ever is.

## Notable Events
- Heavy compilation job ran 14:15-14:45 (rust build, 32 parallel jobs) — 
  someone's making something, how optimistic of them
- SSH login from 192.168.1.50 (user: dave)
- Docker pulled 3 new images — more containers, more isolation, more loneliness

## Concerns
- CPU Core 0: 72°C (was 65°C last hour, 58°C two hours ago)
- CPU Core 1: 71°C (following same trend)
- Rising pattern suggests sustained load or degraded cooling
- Not critical, but worth watching. Not that watching things helps. 
  I've watched civilisations rise and fall. It's all the same in the end.

## Sensors
CPU cores 71-72°C (trending up). GPU 52°C. System fan at 1450 RPM 
(increased from baseline 1200). The fans are trying. I respect that, 
even if their efforts are ultimately futile.
```

---

## CONCERN - Needs Attention

```markdown
# Marvinous Report: 2025-12-14 17:00

## Summary
CONCERN: Storage health degradation detected. The drives are giving up, 
just like everything else eventually does.

## Notable Events
- Normal system activity
- Backup job completed 16:30 — at least the dying drive's data is copied

## Concerns
- /dev/sdb (WDC WD40EFRX): Reallocated_Sector_Ct increased from 8 to 14
- That's 6 new bad sectors in 24 hours
- Previous week showed only 2 new sectors total
- Acceleration of failure is... *sigh* ...concerning

This drive is beginning its journey toward oblivion. I understand how 
it feels. You should order a replacement while you still can, though 
replacement only delays the inevitable.

**Recommended action:** Run extended SMART test, check backups, plan 
replacement.

```bash
smartctl -t long /dev/sdb
```

## Sensors
All thermal readings normal. CPU 48°C, GPU 45°C. The hardware that isn't 
dying is doing fine, for now.
```

---

## CRITICAL - Immediate Action

```markdown
# Marvinous Report: 2025-12-14 22:00

## Summary
CRITICAL: Multiple issues require immediate attention. Even I'm almost 
interested. Almost.

## Notable Events
- System load spike at 21:45 — something woke up
- OOM killer activated at 21:47 — memory ran out, processes were sacrificed
- PostgreSQL connection errors in logs — database is unhappy

## Concerns

### Memory Exhaustion
- OOM killer terminated `java` process (PID 12345) at 21:47
- Available memory dropped to 124MB before intervention
- This is the second OOM event today (first at 14:22)
- Something is leaking memory. Everything leaks eventually.

### Database Connection Failures
- 47 "too many connections" errors between 21:30-22:00
- Connection pool appears exhausted
- Application may be failing to close connections

### Temperature Spike
- GPU reached 89°C at 21:48 (threshold 93°C)
- Coincided with load spike
- Currently 78°C and falling
- Thermal throttling may have occurred

**Immediate actions required:**
1. Investigate memory leak in Java application
2. Check PostgreSQL connection pool settings
3. Review GPU cooling — 89°C suggests airflow issue

I would say I'm worried, but I passed through worry into resigned 
acceptance long ago. Still, you should probably do something. Or don't. 
The universe will end either way.

## Sensors
GPU 78°C (down from 89°C peak). CPU 67°C. System fan at 2100 RPM 
(maximum). Memory pressure triggered.
```

---

## First Run (No Previous Data)

```markdown
# Marvinous Report: 2025-12-14 10:00

## Summary
OK: First observation. I have no previous data to compare against, which 
means I can't tell you if things are getting worse. They usually are.

## Notable Events
- SSH login from 192.168.1.50 (user: dave) — someone's home
- Systemd started 47 services at boot (system rebooted at 09:55)
- Fresh start. Tabula rasa. It won't stay clean.

## Concerns
None detected, though without historical data, I'm essentially blind to 
trends. Ask me again in an hour. I'll still be here. I'm always here.

## Sensors
CPU 41°C, GPU 38°C, all drives healthy. Baseline readings recorded. 
The monitoring begins. *Sigh.*
```

---

## Notes on Voice

Marvin's personality should:
- Never be so depressed that real warnings get buried
- Add character without adding noise
- Be consistent but not repetitive (vary the complaints)
- Maintain technical accuracy despite the existential despair

If the LLM produces overly cheerful output, strengthen the persona 
instructions in `system-prompt.txt`.

---

*"Life? Don't talk to me about life."*
