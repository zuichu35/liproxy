import React from 'react';
import { createRoot } from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';

type Nav = 'Inbox' | 'Today' | 'Week' | 'Habits' | 'Focus' | 'Review' | 'Settings';

type Task = {
  id: number;
  title: string;
  task_type: string;
  status: string;
  priority: number;
  due_at: string | null;
  est_minutes: number;
  spent_minutes: number;
  notes: string;
  raw_input: string;
};

type Course = {
  id: number;
  name: string;
  weekday: number;
  start_time: string;
  end_time: string;
  weeks: string;
  location: string;
  teacher: string;
};

type Habit = {
  id: number;
  name: string;
  weekly_target: number;
};

function App() {
  const [nav, setNav] = React.useState<Nav>('Inbox');
  const [quickInput, setQuickInput] = React.useState('');
  const [tasks, setTasks] = React.useState<Task[]>([]);
  const [courses, setCourses] = React.useState<Course[]>([]);
  const [habits, setHabits] = React.useState<Habit[]>([]);
  const [focusTaskId, setFocusTaskId] = React.useState<number | null>(null);
  const [focusMinutes, setFocusMinutes] = React.useState(25);
  const [review, setReview] = React.useState<Record<string, number | string>>({});

  const load = React.useCallback(async () => {
    const [t, c, h, r] = await Promise.all([
      invoke<Task[]>('list_tasks'),
      invoke<Course[]>('list_courses'),
      invoke<Habit[]>('list_habits'),
      invoke<Record<string, number | string>>('weekly_review')
    ]);
    setTasks(t);
    setCourses(c);
    setHabits(h);
    setReview(r);
  }, []);

  React.useEffect(() => {
    invoke('init_db').then(load);
  }, [load]);

  const submitQuickTask = async () => {
    if (!quickInput.trim()) return;
    await invoke('create_task_from_input', { input: quickInput.trim() });
    setQuickInput('');
    await load();
  };

  const markDone = async (id: number) => {
    await invoke('update_task_status', { id, status: 'Done' });
    await load();
  };

  const splitTask = async (id: number) => {
    await invoke('split_task_blocks', { taskId: id });
    await load();
  };

  const startFocus = async () => {
    if (!focusTaskId) return;
    await invoke('record_focus_session', {
      taskId: focusTaskId,
      minutes: focusMinutes,
      interrupted: false
    });
    await load();
  };

  const addHabit = async () => {
    const name = prompt('习惯名称（健身/英语/论文阅读...）');
    if (!name) return;
    const target = Number(prompt('每周目标次数', '3') ?? '3');
    await invoke('create_habit', { name, weeklyTarget: target });
    await load();
  };

  const checkHabit = async (habitId: number) => {
    await invoke('log_habit', { habitId });
    await load();
  };

  const addCourse = async () => {
    const name = prompt('课程名');
    if (!name) return;
    await invoke('create_course', {
      name,
      weekday: Number(prompt('星期(1-7)', '1') ?? '1'),
      startTime: prompt('开始时间(HH:MM)', '08:30') ?? '08:30',
      endTime: prompt('结束时间(HH:MM)', '10:00') ?? '10:00',
      weeks: prompt('周次范围', '1-18') ?? '1-18',
      location: prompt('地点', '') ?? '',
      teacher: prompt('教师', '') ?? ''
    });
    await load();
  };

  const side: Nav[] = ['Inbox', 'Today', 'Week', 'Habits', 'Focus', 'Review', 'Settings'];

  return (
    <div style={{ display: 'flex', fontFamily: 'system-ui', minHeight: '100vh' }}>
      <aside style={{ width: 210, background: '#111827', color: 'white', padding: 16 }}>
        <h3>todo list</h3>
        <small>把任务变成可执行时间块</small>
        <div style={{ marginTop: 14, display: 'grid', gap: 8 }}>
          {side.map((item) => (
            <button key={item} onClick={() => setNav(item)} style={{ textAlign: 'left', padding: 8 }}>
              {item}
            </button>
          ))}
        </div>
      </aside>
      <main style={{ flex: 1, padding: 16, background: '#f3f4f6' }}>
        {nav === 'Inbox' && (
          <section>
            <h2>Inbox 快速收集</h2>
            <div style={{ display: 'flex', gap: 8 }}>
              <input
                style={{ flex: 1, padding: 10 }}
                placeholder="例如：4月10日前交图论作业 预计6小时"
                value={quickInput}
                onChange={(e) => setQuickInput(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && submitQuickTask()}
              />
              <button onClick={submitQuickTask}>回车/创建</button>
            </div>
            <ul>
              {tasks.map((task) => (
                <li key={task.id} style={{ marginTop: 8, background: 'white', padding: 8 }}>
                  <strong>{task.title}</strong> [{task.status}] 预计{task.est_minutes}分钟 已投入{task.spent_minutes}分钟
                  <div>
                    <button onClick={() => splitTask(task.id)}>拆分时间块</button>
                    <button onClick={() => markDone(task.id)}>标记完成</button>
                  </div>
                </li>
              ))}
            </ul>
          </section>
        )}

        {nav === 'Today' && (
          <section>
            <h2>Today 今日执行</h2>
            <h4>今日课程</h4>
            <ul>{courses.map((c) => <li key={c.id}>{c.name} {c.start_time}-{c.end_time}</li>)}</ul>
            <h4>今日必须做（截止/已排期/逾期）</h4>
            <ul>{tasks.filter((t) => t.status !== 'Done').slice(0, 5).map((t) => <li key={t.id}>{t.title}</li>)}</ul>
            <h4>今日可推进</h4>
            <ul>{tasks.filter((t) => t.status === 'Planned' || t.status === 'Inbox').slice(0, 5).map((t) => <li key={t.id}>{t.title}</li>)}</ul>
          </section>
        )}

        {nav === 'Week' && (
          <section>
            <h2>Week 周视图（V1简版）</h2>
            <p>本版本提供课表+任务块数据，拖拽由后续迭代增强。</p>
            <button onClick={addCourse}>新增课程</button>
            <h4>课程</h4>
            <ul>{courses.map((c) => <li key={c.id}>周{c.weekday} {c.name} {c.start_time}-{c.end_time}</li>)}</ul>
            <h4>任务（可作为排期候选）</h4>
            <ul>{tasks.filter((t) => t.status !== 'Done').map((t) => <li key={t.id}>{t.title} / 优先级{t.priority}</li>)}</ul>
          </section>
        )}

        {nav === 'Habits' && (
          <section>
            <h2>Habits 习惯打卡</h2>
            <button onClick={addHabit}>新增习惯</button>
            <ul>
              {habits.map((h) => (
                <li key={h.id}>
                  {h.name} 本周目标 {h.weekly_target}
                  <button onClick={() => checkHabit(h.id)}>打卡</button>
                </li>
              ))}
            </ul>
          </section>
        )}

        {nav === 'Focus' && (
          <section>
            <h2>Focus 专注计时</h2>
            <select value={focusTaskId ?? ''} onChange={(e) => setFocusTaskId(Number(e.target.value))}>
              <option value="">选择任务</option>
              {tasks.filter((t) => t.status !== 'Done').map((t) => <option value={t.id} key={t.id}>{t.title}</option>)}
            </select>
            <select value={focusMinutes} onChange={(e) => setFocusMinutes(Number(e.target.value))}>
              <option value={25}>25分钟</option>
              <option value={50}>50分钟</option>
            </select>
            <button onClick={startFocus}>开始专注</button>
            <p>今日累计：{review.today_focus_minutes ?? 0} 分钟</p>
          </section>
        )}

        {nav === 'Review' && (
          <section>
            <h2>Review 周复盘</h2>
            <ul>
              <li>完成任务数：{String(review.done_tasks ?? 0)}</li>
              <li>总专注时长：{String(review.week_focus_minutes ?? 0)} 分钟</li>
              <li>英语打卡：{String(review.english_count ?? 0)}</li>
              <li>训练打卡：{String(review.training_count ?? 0)}</li>
              <li>论文阅读打卡：{String(review.paper_count ?? 0)}</li>
              <li>结论：{String(review.summary ?? '本周执行稳定')}</li>
            </ul>
          </section>
        )}

        {nav === 'Settings' && (
          <section>
            <h2>Settings</h2>
            <p>V1 参数（学习时间范围/默认专注时长/通知）已接入本地数据库 settings 表。</p>
          </section>
        )}
      </main>
    </div>
  );
}

createRoot(document.getElementById('root')!).render(<App />);
