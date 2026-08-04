import { defineBoot } from "#q-app/wrappers";
import { getToken } from "@/api/client";
import { useAuthStore } from "@/stores/auth";

export default defineBoot(async ({ store }) => {
  const auth = useAuthStore(store);
  auth.restore();
  if (getToken()) await auth.fetchProfile();
});
