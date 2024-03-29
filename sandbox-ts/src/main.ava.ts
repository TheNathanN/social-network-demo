import { Worker, NearAccount } from "near-workspaces"
import anyTest, { TestFn } from "ava"

const test = anyTest as TestFn<{
  worker: Worker
  accounts: Record<string, NearAccount>
}>

test.beforeEach(async (t) => {
  // Init the worker and start a Sandbox server
  const worker = await Worker.init()

  // Deploy contract
  const root = worker.rootAccount
  const contract = await root.createSubAccount("test-account")
  // Get wasm file path from package.json test script in folder above
  await contract.deploy(process.argv[2])

  // Save state for test runs, it is unique for each test
  t.context.worker = worker
  t.context.accounts = { root, contract }
})

test.afterEach.always(async (t) => {
  // Stop Sandbox server
  await t.context.worker.tearDown().catch((error) => {
    console.log("Failed to stop the Sandbox:", error)
  })
})

test("creates a new post", async (t) => {
  const { root, contract } = t.context.accounts

  const post: {
    title: string
    tags: string[]
  } = await root.call(contract, "add_post", {
    title: "Test",
    description: "Test Description",
    tags: "tag1,tag2,tag3",
    media: "post.png",
  })

  t.is(post.title, "Test")
  t.is(post.tags[2], "tag3")
})

test("gets all posts", async (t) => {
  const { root, contract } = t.context.accounts

  await root.call(contract, "add_post", {
    title: "Test0",
    description: "Test Description0",
    tags: "tag1,tag2,tag3",
    media: "post.png",
  })
  await root.call(contract, "add_post", {
    title: "Test1",
    description: "Test Description1",
    tags: "tag4,tag5,tag6",
    media: "post.png",
  })
  await root.call(contract, "add_post", {
    title: "Test2",
    description: "Test Description2",
    tags: "tag1,tag5,tag7",
    media: "post.png",
  })

  const allPosts: any = await contract.view("get_all_posts")

  t.is(allPosts[1][1].title, "Test1")
  t.is(allPosts[2][1].description, "Test Description2")
})
