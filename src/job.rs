use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PidWrapper(pub i32);

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Running,
    #[allow(dead_code)]
    Stopped,
    Done,
    Terminated,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: u32,
    pub pid: PidWrapper,
    pub command: String,
    pub status: JobStatus,
    pub foreground: bool,
}

pub struct JobControl {
    jobs: HashMap<u32, Job>,
    next_id: u32,
}

impl JobControl {
    pub fn new() -> Self {
        JobControl {
            jobs: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_job(&mut self, pid: PidWrapper, command: String, foreground: bool) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.jobs.insert(
            id,
            Job {
                id,
                pid,
                command,
                status: JobStatus::Running,
                foreground,
            },
        );
        id
    }

    pub fn update_job(&mut self, id: u32, status: JobStatus) {
        if let Some(job) = self.jobs.get_mut(&id) {
            job.status = status;
        }
    }

    pub fn remove_job(&mut self, id: u32) -> Option<Job> {
        self.jobs.remove(&id)
    }

    pub fn get_job(&self, id: u32) -> Option<&Job> {
        self.jobs.get(&id)
    }

    pub fn list_jobs(&self) -> Vec<&Job> {
        let mut jobs: Vec<&Job> = self.jobs.values().collect();
        jobs.sort_by_key(|j| j.id);
        jobs
    }

    pub fn cleanup_done(&mut self) {
        self.jobs
            .retain(|_, j| j.status != JobStatus::Done && j.status != JobStatus::Terminated);
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    pub fn find_by_pid(&self, pid: i32) -> Option<u32> {
        self.jobs
            .iter()
            .find(|(_, j)| j.pid.0 == pid)
            .map(|(id, _)| *id)
    }
}

pub fn format_job_status(job: &Job) -> String {
    let status_str = match job.status {
        JobStatus::Running => "Running",
        JobStatus::Stopped => "Stopped",
        JobStatus::Done => "Done",
        JobStatus::Terminated => "Terminated",
    };
    format!("[{}] {}  {}", job.id, status_str, job.command)
}
