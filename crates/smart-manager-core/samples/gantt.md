```mermaid
---
config:
  theme: base
  themeVariables:
    taskBkgColor: '#1f3a5f'
    taskBorderColor: '#000000'
    taskTextColor: '#ffffff'
    taskTextLightColor: '#ffffff'
    taskTextDarkColor: '#ffffff'
    taskTextOutsideColor: '#ffffff'
    critBkgColor: '#7a1f1f'
    critBorderColor: '#000000'
    activeTaskBkgColor: '#1f5a4a'
    activeTaskBorderColor: '#000000'
    doneTaskBkgColor: '#444444'
    doneTaskBorderColor: '#666666'
    sectionBkgColor: '#2a2a2a'
    altSectionBkgColor: '#1a1a1a'
    sectionBkgColor2: '#3a3a3a'
    gridColor: '#888888'
    titleColor: '#ffffff'
    textColor: '#ffffff'
    primaryTextColor: '#ffffff'
    todayLineColor: '#ff6b6b'
---
gantt
    title Schedule
    dateFormat YYYY-MM-DD
    axisFormat %m-%d
    section Ship MVP
    [Q2] Who is the pilot user? :crit, t0, 2025-01-01, 2d
    [Q0] Is the data model right? :t1, after t0, 3d
    [Q1] Which platforms first? :t2, after t1, 2d
    section Refresh public docs
    [Q0] Are docs current? :t3, 2025-01-01, 3d
    section Learn type theory
    [Q0] What's the right textbook? :t4, 2025-01-01, 3d
```

## Reference

### Ship MVP

| ID | Question | Priority | Days | Status | Action points |
|----|----------|----------|------|--------|---------------|
| Q2 | Who is the pilot user? | Critical | 2 | open | [ ] Outreach to candidates (2d, managing) |
| Q0 | Is the data model right? | High | 3 | open | [ ] Design schema (2d, analysis)<br>[ ] Write migration (1d, programming) |
| Q1 | Which platforms first? | Medium | 2 | open | [ ] Compare desktop vs web (1.5d, research)<br>[ ] Decision doc (0.5d, writing) |

### Refresh public docs

| ID | Question | Priority | Days | Status | Action points |
|----|----------|----------|------|--------|---------------|
| Q0 | Are docs current? | Low | 3 | open | [x] Audit landing page (1d, qa)<br>[ ] Rewrite quickstart (2d, writing) |

### Learn type theory

| ID | Question | Priority | Days | Status | Action points |
|----|----------|----------|------|--------|---------------|
| Q0 | What's the right textbook? | LongTerm | 3 | open | [ ] Read TaPL ch.1 (3d, research) |
