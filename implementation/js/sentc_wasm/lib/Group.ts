/**
 * @author Jörn Heinemann <joernheinemann@gmx.de>
 * @since 2022/08/12
 */
import {GroupData, GroupJoinReqListItem, GroupKey, GroupKeyRotationOut, USER_KEY_STORAGE_NAMES} from "./Enities";
import {
	group_accept_join_req, group_done_key_rotation, group_finish_key_rotation, group_get_group_keys,
	group_get_join_reqs,
	group_invite_user,
	group_invite_user_session,
	group_join_user_session, group_key_rotation, group_pre_done_key_rotation, group_prepare_key_rotation,
	group_prepare_keys_for_new_member,
	group_reject_join_req, leave_group
} from "../pkg";
import {Sentc} from "./Sentc";


export class Group
{
	constructor(private data: GroupData, private base_url: string, private app_token: string) {}

	//__________________________________________________________________________________________________________________

	public prepareKeysForNewMember(user_id: string)
	{
		const key_count = this.data.keys.length;

		//TODo get or fetch the user public data via static fn in sentc

		const key_string = JSON.stringify(this.data.keys);

		return group_prepare_keys_for_new_member(user_id, key_string, key_count, this.data.rank);
	}

	public async invite(user_id: string)
	{
		//TODO get or fetch the user public data via static fn in sentc

		const key_count = this.data.keys.length;
		const [key_string] = this.prepareKeyString();

		const jwt = await Sentc.getJwt();

		const session_id = await group_invite_user(
			this.base_url,
			this.app_token,
			jwt,
			this.data.group_id,
			user_id,
			key_count,
			this.data.rank,
			user_id,	//TODO use the public key here
			key_string
		);

		if (session_id === "") {
			return;
		}

		//upload the rest of the keys via session
		let next_page = true;
		let i = 1;
		const p = [];

		while (next_page) {
			const next_keys = this.prepareKeyString(i);
			next_page = next_keys[1];

			p.push(group_invite_user_session(
				this.base_url,
				this.app_token,
				jwt,
				this.data.group_id,
				session_id,
				user_id, //TODo use the public key here
				next_keys[0]
			));

			i++;
		}

		return Promise.allSettled(p);
	}

	//__________________________________________________________________________________________________________________
	//join req

	public async getJoinRequests(last_fetched_item: GroupJoinReqListItem | null = null)
	{
		const jwt = await Sentc.getJwt();

		const reqs: GroupJoinReqListItem[] = await group_get_join_reqs(
			this.base_url,
			this.app_token,
			jwt,
			this.data.group_id,
			this.data.rank,
			last_fetched_item.time.toString(),
			last_fetched_item.user_id
		);

		return reqs;
	}

	public async rejectJoinRequest(user_id: string)
	{
		const jwt = await Sentc.getJwt();

		return group_reject_join_req(
			this.base_url,
			this.app_token,
			jwt,
			this.data.group_id,
			this.data.rank,
			user_id
		);
	}

	public async acceptJoinRequest(user_id: string)
	{
		const jwt = await Sentc.getJwt();
		const key_count = this.data.keys.length;
		const [key_string] = this.prepareKeyString();

		//TODO get or fetch the user public data via static fn in sentc

		const session_id = await group_accept_join_req(
			this.base_url,
			this.app_token,
			jwt,
			this.data.group_id,
			user_id,
			key_count,
			this.data.rank,
			user_id,	//TODo use the public key here
			key_string
		);

		if (session_id === "") {
			return;
		}

		let next_page = true;
		let i = 1;
		const p = [];

		while (next_page) {
			const next_keys = this.prepareKeyString(i);
			next_page = next_keys[1];

			p.push(group_join_user_session(
				this.base_url,
				this.app_token,
				jwt,
				this.data.group_id,
				session_id,
				user_id, //TODo use the public key here
				next_keys[0]
			));

			i++;
		}

		return Promise.allSettled(p);
	}

	//__________________________________________________________________________________________________________________

	public async leave()
	{
		const jwt = await Sentc.getJwt();

		return leave_group(
			this.base_url,
			this.app_token,
			jwt,
			this.data.group_id
		);
	}

	//__________________________________________________________________________________________________________________
	//key rotation

	public async prepareKeyRotation()
	{
		const user = await Sentc.getActualUser(true);

		return group_prepare_key_rotation(this.data.keys[this.data.keys.length - 1].group_key, user.public_key);
	}

	public async doneKeyRotation(server_output: string)
	{
		const user = await Sentc.getActualUser(true);

		return group_done_key_rotation(user.private_key, user.public_key, this.data.keys[this.data.keys.length - 1].group_key, server_output);
	}

	public async keyRotation()
	{
		const user = await Sentc.getActualUser(true);

		return group_key_rotation(this.base_url, this.app_token, user.jwt, this.data.group_id, user.public_key, this.data.keys[this.data.keys.length - 1].group_key);
	}

	public async finishKeyRotation()
	{
		const user = await Sentc.getActualUser(true);

		let keys: GroupKeyRotationOut[] = await group_pre_done_key_rotation(this.base_url, this.app_token, user.jwt, this.data.group_id);

		let next_round = false;
		let rounds_left = 10;

		do {
			const left_keys = [];

			//should be always there because the group rotation keys are ordered by time
			for (let i = 0; i < keys.length; i++) {
				const key = keys[i];

				const pre_key_index = this.data.key_map.get(key.pre_group_key_id);
				if (!pre_key_index) {
					left_keys.push(key);
					continue;
				}

				const pre_key = this.data.keys[pre_key_index];
				if (!pre_key) {
					left_keys.push(key);
					continue;
				}

				//await must be in this loop because we need the keys
				// eslint-disable-next-line no-await-in-loop
				await group_finish_key_rotation(
					this.base_url,
					this.app_token,
					user.jwt,
					this.data.group_id,
					key.server_output,
					pre_key.group_key,
					user.public_key,
					user.private_key
				);
			}

			//when it runs 10 times and there are still left -> break up
			rounds_left--;

			//fetch the new keys, when there are still keys left, maybe they are there after the key fetch -> must be in loop too
			// eslint-disable-next-line no-await-in-loop
			await this.fetchKeys(user.jwt, user.private_key);

			if (left_keys.length > 0) {
				keys = [];
				//push the not found keys into the key array, maybe the pre group keys are in the next round
				keys.push(...left_keys);

				next_round = true;
			} else {
				next_round = false;
			}
		} while (next_round && rounds_left > 0);

		//after a key rotation -> save the new group data in the store
		const storage = await Sentc.getStore();
		const group_key = USER_KEY_STORAGE_NAMES.groupData + "_id_" + this.data.group_id;
		return storage.set(group_key, this.data);
	}

	//__________________________________________________________________________________________________________________

	public async fetchKeys(jwt: string, private_key: string)
	{
		let last_item = this.data.keys[this.data.keys.length - 1];

		let next_fetch = true;

		const keys: GroupKey[] = [];

		while (next_fetch) {
			// eslint-disable-next-line no-await-in-loop
			const fetchedKeys: GroupKey[] = await group_get_group_keys(
				this.base_url,
				this.app_token,
				jwt,
				private_key,
				this.data.group_id,
				last_item.time.toString(),
				last_item.group_key_id
			);

			keys.push(...fetchedKeys);

			next_fetch = fetchedKeys.length > 50;

			last_item = fetchedKeys[fetchedKeys.length - 1];
		}

		const last_inserted_key_index = this.data.keys.length;

		//insert in the key map
		for (let i = 0; i < keys.length; i++) {
			this.data.key_map.set(keys[i].group_key_id, i + last_inserted_key_index);
		}

		this.data.keys.push(...keys);

		//return the updated data, so it can be saved in the store
		return this.data;
	}

	private prepareKeyString(page = 0): [string, boolean]
	{
		const offset = page * 50;
		const end = (offset + 50 > this.data.keys.length - 1) ? this.data.keys.length - 1 : offset + 50;

		const key_slice = this.data.keys.slice(offset, end);

		const keys = [];

		for (let i = 0; i < key_slice.length; i++) {
			const key = this.data.keys[i].group_key;

			keys.push(key);
		}

		return [JSON.stringify(keys), end < this.data.keys.length - 1];
	}
}
