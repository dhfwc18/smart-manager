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
    doneTaskBkgColor: '#1f3a5f'
    doneTaskBorderColor: '#000000'
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
    section work
    Ship MVP :crit, t0, 2025-01-01, 7d
    Refresh public docs :t1, after t0, 2d
    section personal
    Learn type theory :t2, 2025-01-01, 3d
```
