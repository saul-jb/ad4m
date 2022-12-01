import { expect } from "chai";
import { exec, execSync, ChildProcess } from 'node:child_process';
import { GraphQLWsLink } from "@apollo/client/link/subscriptions/index.js";
import { ApolloClient, InMemoryCache } from "@apollo/client/core/index.js";
import { HttpLink } from "@apollo/client/link/http/index.js";
import Websocket from "ws";
import { createClient } from "graphql-ws";
import { Ad4mClient, Link, Literal, PerspectiveProxy, Subject } from "@perspect3vism/ad4m";
import { rmSync, readFileSync } from "node:fs";
import fetch from 'node-fetch';


function apolloClient(port: number, token?: string): ApolloClient<any> {
    const wsLink = new GraphQLWsLink(createClient({
        url: `ws://localhost:${port}/graphql`,
        webSocketImpl: Websocket,
    }));

    const link = new HttpLink({
        uri: "http://localhost:4000/graphql",
        //@ts-ignore
        fetch
      });
  
    return new ApolloClient({
        link: wsLink,
        cache: new InMemoryCache({ resultCaching: false, addTypename: false }),
        defaultOptions: {
            watchQuery: {
                fetchPolicy: "no-cache",
            },
            query: {
                fetchPolicy: "no-cache",
            }
        },
    });
  }

describe("Integration", () => {
    let ad4m: Ad4mClient | null = null
    let executorProcess: ChildProcess | null = null

    before(async () => {
        ad4m = new Ad4mClient(apolloClient(4000))
        return
        /*
        rmSync("../ad4mJS", { recursive: true, force: true })
        console.log("Initialzing executor data directory")
        //@ts-ignore
        execSync('../../host/dist/ad4m-macos-x64 init --dataPath ../ad4mJS', {})
        
        console.log("Starting executor")
        execSync("killall holochain")
        //@ts-ignore
        executorProcess = exec('../../host/dist/ad4m-macos-x64 serve --dataPath ../ad4mJS', {})

        let executorReady = new Promise<void>((resolve, reject) => {
            executorProcess!.stdout!.on('data', (data) => {
                if (data.includes("GraphQL server started")) {
                    resolve()
                }
            });
        })

        executorProcess!.stdout!.on('data', (data) => {
            console.log(`${data}`);
        });
        //Log holochain process stderr to out
        executorProcess!.stderr!.on('data', (data) => {
            console.log(`${data}`);
        });
    
        console.log("Waiting for executor to settle...")
        await executorReady
        console.log("Creating ad4m client")
        ad4m = new Ad4mClient(apolloClient(4000))
        console.log("Generating agent")
        await ad4m.agent.generate("secret")
        console.log("Done")
        */
    })

    after(() => {
        if (executorProcess) {
            executorProcess.kill()
        }
    })

    it("should get agent status", async () => {
        let result = await ad4m!.agent.status()
        expect(result).to.not.be.null
        expect(result!.isInitialized).to.be.true
    })

    describe("Subjects", () => {
        let perspective: PerspectiveProxy | null = null

        before(async () => {
            perspective = await ad4m!.perspective.add("test")
            // for test debugging:
            console.log("UUID: " + perspective.uuid)

            let classes = await perspective.subjectClasses();
            expect(classes.length).to.equal(0)

            let sdna = readFileSync("./subject.pl").toString()
            await perspective.setSdna(sdna)

            let retrievedSdna = await perspective.getSdna()
            expect(retrievedSdna).to.deep.equal([sdna])
        })

        it("should find the TODO subject class from the test SDNA", async () => {
            let classes = await perspective!.subjectClasses();

            expect(classes.length).to.equal(1)
            expect(classes[0]).to.equal("TODO")
        })

        it("should be able to construct a subject instance from a literal", async () => {
            let root = Literal.from("construct test").toUrl()
            expect(await perspective!.constructSubject("TODO", root)).to.be.true
            expect(await perspective!.isSubjectInstance(root, "TODO")).to.be.true
        })
    
        describe("with an instance", () => {
            let subject: Subject | null = null

            before(async () => {
                let root = Literal.from("construct test").toUrl()
                await perspective!.constructSubject("TODO", root)
                subject = await perspective!.subjectInstance(root, "TODO")
            })

            it("should be able to read a property as JS property", async () => {
                //@ts-ignore
                expect(await subject.state).to.equal("todo://ready")
            })

            it("should be able to set a property with JS setter method", async () => {
                //@ts-ignore
                await subject.setState("todo://done")
                //@ts-ignore
                expect(await subject.state).to.equal("todo://done")
            })

            it("should work with a property that is not set initially", async () => {
                //@ts-ignore
                expect(await subject.title).to.be.undefined
        
                let title = Literal.from("test title").toUrl()
                //@ts-ignore
                await subject.setTitle(title)
                //@ts-ignore
                expect(await subject.title).to.equal(title)
            })

            it("should be able to get collections as arrays", async () => {
                //@ts-ignore
                expect(await subject.comments).to.be.an("array")
                //@ts-ignore
                expect(await subject.comments).to.be.empty

                let c1 = Literal.from("comment 1").toUrl()
                await perspective!.add(new Link({
                    source: subject!.baseExpression,
                    predicate: "todo://comment", 
                    target: c1
                }))

                //@ts-ignore
                expect(await subject.comments).to.deep.equal([c1])

                let c2 = Literal.from("comment 2").toUrl()
                await perspective!.add(new Link({
                    source: subject!.baseExpression,
                    predicate: "todo://comment", 
                    target: c2
                }))

                //@ts-ignore
                expect(await subject.comments).to.deep.equal([c1, c2])
            })
        })
    })


})