import "learning-service.dsl" as learning

module quizService {
  context: engagement.quiz
  owner: team.engagement

  container api: Service "Quiz API"
  api -> learning.api
}