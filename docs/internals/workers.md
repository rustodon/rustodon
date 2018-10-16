okay so this is how jobs are gonna work
* spawn "dispatch" thread
    * this boy just sits there in a nice little loop {}
    * tries to grab N jobs from the db
        * if it can't, just yields its time slice
    * figure out what jobs we should run _now_ (`can_run: Vec<&dyn Job>`)
    * ```sql
        UPDATE jobs SET jobstatus = INFLIGHT, updated = #{time.now} WHERE id IN can_run
      ```
    * push into thread queue
* spawn "timeout" thread
    * also doing a loop {}
    * ```sql
        SELECT jobid FROM jobs WHERE jobstatus = INFLIGHT, "updated" + "timeout" < #{time.now}
      ```
    * do we need to kill inflight jobs?
