(*
 * Copyright (c) 2018, Facebook, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE fn in the "hack" directory of this source tree.
 *
 *)

(* Functions used by monitor *)

type pipe_from_server

val make_pipe_from_server : Unix.file_descr -> pipe_from_server

val read_from_server :
  pipe_from_server -> MonitorRpc.server_to_monitor_message option

(* Functions used by server *)

(* This pipe is global per server process. There is only one monitor per server
 * lifetime, and the server wants to use this pipe in a lot of places where it's
 * annoying to thread it too *)
val make_pipe_to_monitor : Unix.file_descr -> unit

val send_to_monitor : MonitorRpc.server_to_monitor_message -> unit

(* This is basically signature of "Printf.printf" *)
val send_progress_to_monitor :
  ?include_in_logs:bool -> ('a, unit, string, unit) format4 -> 'a

(* The message will look roughly like this:
  <operation> <done_count>/<total_count> <unit> <percent done>*)
val make_percentage_progress_message :
  operation:string ->
  done_count:int ->
  total_count:int ->
  unit:string ->
  string

val send_percentage_progress_to_monitor :
  operation:string -> done_count:int -> total_count:int -> unit:string -> unit
