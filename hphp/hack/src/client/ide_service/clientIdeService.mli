(*
 * Copyright (c) 2019, Facebook, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the "hack" directory of this source tree.
 *
 *)

(** Provides IDE services in the client, without an instance of hh_server
running.

Basic approach: we load the naming table to give us just enough symbol
information to provide IDE services for just the files you're looking at. When
we need to look up declarations to service an IDE query, we parse and typecheck
the files containing those declarations on-demand, then answer your IDE query.
*)
type t

module Status : sig
  type t =
    | Not_started
        (** The IDE services haven't been requested to initialize yet. *)
    | Initializing
        (** The IDE services are still initializing (e.g. loading saved-state or
        building indexes.) *)
    | Processing_files of ClientIdeMessage.Processing_files.t
        (** The IDE services are available, but are also in the middle of
        processing files. *)
    | Ready  (** The IDE services are available. *)
    | Stopped of string  (** The IDE services are not available. *)
end

module Stop_reason : sig
  type t =
    | Crashed
    | Editor_exited
    | Restarting
    | Testing

  val to_string : t -> string
end

(** Create an uninitialized IDE service. All queries made to this service will
fail immediately, unless otherwise requested in the initialization procedure. *)
val make : unit -> t

(** Request that the IDE service initialize from the saved state. Queries made
to the service will fail until it is done initializing, unless
[wait_for_initialization] is [true], in which case queries made to the service
will block until the initializing is complete. *)
val initialize_from_saved_state :
  t ->
  root:Path.t ->
  naming_table_saved_state_path:Path.t option ->
  wait_for_initialization:bool ->
  use_ranked_autocomplete:bool ->
  (unit, string) Lwt_result.t

(** Pump the message loop for the IDE service. Exits once the IDE service has
been [destroy]ed. *)
val serve : t -> unit Lwt.t

(** Clean up any resources held by the IDE service (such as the message loop
and background processes). Mark the service's status as "shut down" for the
given [reason]. *)
val stop : t -> reason:Stop_reason.t -> unit Lwt.t

(** The caller is expected to call this function to notify the IDE service
whenever a Hack file changes on disk, so that it can update its indexes
appropriately. *)
val notify_file_changed : t -> Path.t -> unit

(** Make an RPC call to the IDE service. *)
val rpc : t -> 'response ClientIdeMessage.t -> ('response, string) Lwt_result.t

(** Get a handle to the stream of notifications sent by the IDE service. These
notifications may be sent even during RPC requests, and so should be processed
asynchronously. *)
val get_notifications : t -> ClientIdeMessage.notification Lwt_message_queue.t

(** Get the status of the IDE services, based on the internal state and any
notifications that the IDE service process has sent to us. *)
val get_status : t -> Status.t
