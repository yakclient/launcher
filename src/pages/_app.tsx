import type {AppProps} from 'next/app'
import 'bootstrap/dist/css/bootstrap.min.css'
import './globals.scss'
import {Alert, ProgressBar, ThemeProvider} from "react-bootstrap";
import React, {ReactNode, useContext, useEffect, useState} from "react";
import {Variant} from "react-bootstrap/types";
import {Channel, invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";

// ---- ALERT SYSTEM ----

const ALERT_TIMEOUT = 10

export interface LauncherAlert {
    variant: Variant,
    content: React.ReactNode,
    id: number
}

type AddAlert = (variant: Variant, content: ReactNode) => void

export const Alerts = React.createContext<AddAlert>(() => {});

// ---- CONSOLE LOG SYSTEM ----

export type ConsoleLine = {
    is_err: string,
    frag: number[]
}

type CCContext = {
    channel: Channel<ConsoleLine> | undefined,
    setChannel: (channel: Channel<ConsoleLine>) => void
};

export const ConsoleChannel = React.createContext<CCContext>({
    channel: undefined,
    setChannel: () => {}
})

export const useConsole = () => useContext(ConsoleChannel)

// ---- APP ENTRY ----

interface Task {
    event: TaskEvent,
    progress: number,
    err: string | null,
    channel: Channel<TaskProgressData>
}

interface TaskEvent {
    name: string,
    id: number,
}

interface TaskProgressData {
    progress: number,
    error: string | null,
}

export default function MyApp({Component, pageProps}: AppProps) {
    const [alertCount, setAlertCount] = useState(0);
    const [alerts, setAlerts] = useState<LauncherAlert[]>([]);
    const [channel, setChannel] = useState<Channel<ConsoleLine> | undefined>(undefined)

    const addAlert: AddAlert = (variant: Variant, content: ReactNode) => {
        let id = alertCount + 1;
        const alert = {
            variant: variant,
            content: content,
            id: id
        }

        setAlertCount(id)

        alerts.push(alert)

        setTimeout(() => {
            setAlerts((newAlerts) => newAlerts.filter((t) => t.id != id))
        }, ALERT_TIMEOUT * 1000);
    }

    const [tasks, setTasks] = useState<Task[]>([])

    useEffect(() => {
        let unlisten = listen<TaskEvent>("new-task", ({payload}) => {
            const channel = new Channel<TaskProgressData>()
            const task: Task = {
                event: payload,
                progress: 0,
                err: null,
                channel: channel
            }

            channel.onmessage = (update: TaskProgressData) => {
                setTasks((prev) => {
                    return prev.map((it) => {
                        if (it.event.id == task.event.id) {

                            it.progress = update.progress

                            if (update.error)
                                it.err = update.error

                            if (Math.floor(update.progress) == 1) {
                                setTimeout(() => {
                                    setTasks((prev1) => prev1.filter((t) => {
                                        return t.event.id != task.event.id
                                    }))
                                }, it.err ? 4000 : 1000)
                            }
                        }

                        return it
                    })
                })
            }

            invoke("register_task_channel", {
                id: task.event.id,
                channel: channel
            }).then(() => {
                setTasks((prev) => {
                    return [...prev, task]
                })
            })
        }).catch(() => {})

        return () => {
            // @ts-ignore
            unlisten.then(remove => remove ? remove() : {})
        };
    })

    return <div data-bs-theme="dark">
        <ThemeProvider>
            <ConsoleChannel.Provider value={{
                channel,
                setChannel
            } as CCContext}>
                <Alerts.Provider value={addAlert}>
                    <Component {...pageProps} />
                    <div
                        style={{
                            position: "absolute",
                            bottom: 0,
                            right: 0,
                            zIndex: 20,
                            margin: "10px",
                            transition: "ease-in"
                        }}
                    >
                        {alerts.map((value, index) =>
                            <Alert key={index} variant={value.variant} dismissible>
                                {value.content}
                            </Alert>
                        )}
                        {tasks.map((value) => {
                            console.log(value.progress * 100)
                            return <Alert key={value.event.id} variant={"dark"}>
                                <h2>{value.event.name}</h2>
                                <ProgressBar style={{
                                    margin: "10px 0"
                                }} animated now={value.progress * 100} variant={value.err ? "danger" : "success"}/>
                                <i>{
                                    value.err ?? (Math.floor(value.progress * 100) + "% Done")
                                }</i>
                            </Alert>
                        })}
                    </div>
                </Alerts.Provider>
            </ConsoleChannel.Provider>
        </ThemeProvider>
    </div>
}

